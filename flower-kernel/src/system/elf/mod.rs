use alloc::string::{String, ToString};
use alloc::vec::Vec;

use x86_64::structures::paging::PageTableFlags;
use x86_64::{VirtAddr, align_down, align_up};
use xmas_elf::program::{Flags, Type};
use xmas_elf::{ElfFile, header};

use crate::arch;
use crate::arch::layout::USER_DYNAMIC_LINKER_BASE;
use crate::system::mem::vmm::AddressSpace;

#[derive(Debug)]
pub struct ELF64 {
    pub entry: u64,
    pub size: usize,

    pub phdr: u64,
    pub phent: u64,
    pub phnum: u64,

    pub interp: Option<String>,
    pub base: u64,

    pub end: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ELFLoadType {
    Main,
    Interpreter,
}

fn merge_page_flags(
    current: PageTableFlags,
    requested: PageTableFlags,
) -> PageTableFlags {
    let any_exec = !current.contains(PageTableFlags::NO_EXECUTE)
        || !requested.contains(PageTableFlags::NO_EXECUTE);

    let mut merged = current | requested;
    if any_exec {
        merged.remove(PageTableFlags::NO_EXECUTE);
    }

    merged
}

fn segment_flags(flags: Flags) -> PageTableFlags {
    let mut result = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;

    if flags.is_write() {
        result |= PageTableFlags::WRITABLE;
    }

    if !flags.is_execute() {
        result |= PageTableFlags::NO_EXECUTE;
    }

    result
}

pub fn load_into(
    elf_data: &[u8],
    address_space: &AddressSpace,
    load_type: ELFLoadType,
) -> Result<ELF64, &'static str> {
    let elf = ElfFile::new(elf_data).map_err(|_| "failed to read elf")?;
    let header = elf.header;

    if header.pt2.machine().as_machine() != header::Machine::X86_64 {
        return Err("not amd64");
    }

    let load_base: u64 = match (header.pt2.type_().as_type(), load_type) {
        (header::Type::Executable, _) => 0x0,
        (header::Type::SharedObject, ELFLoadType::Main) => 0x0,
        (header::Type::SharedObject, ELFLoadType::Interpreter) => {
            USER_DYNAMIC_LINKER_BASE
        },
        _ => return Err("unsupported elf type"),
    };

    let mut interp: Option<String> = None;
    let mut image_end = load_base;

    for program_header in elf.program_iter() {
        let align = program_header.align();

        if align > 1
            && program_header.virtual_addr() % align
                != program_header.offset() % align
        {
            return Err("bad segment alignment");
        }

        // load segments
        match program_header.get_type().map_err(|_| "invalid header")? {
            Type::Load => {
                let flags = segment_flags(program_header.flags());

                let vaddr = program_header.virtual_addr();
                let mem_size = program_header.mem_size();

                let start_page = align_down(
                    vaddr + load_base,
                    arch::layout::PAGE_SIZE as u64,
                );
                let end_page = align_up(
                    vaddr + mem_size + load_base,
                    arch::layout::PAGE_SIZE as u64,
                );

                if end_page > image_end {
                    image_end = end_page;
                }

                let mut addr = start_page;

                while addr < end_page {
                    let page = VirtAddr::new(addr);

                    if !address_space.is_mapped(page) {
                        address_space.map_page_alloc(page, flags)?;
                    } else {
                        let current = address_space.page_flags(page)?;
                        let merged = merge_page_flags(current, flags);
                        address_space.update_page_flags(page, merged)?;
                    }

                    addr += arch::layout::PAGE_SIZE as u64;
                }

                let offset = program_header.offset() as usize;
                let file_size = program_header.file_size() as usize;
                let file_end = offset
                    .checked_add(file_size)
                    .ok_or("invalid segment size")?;

                if file_end > elf_data.len() {
                    return Err("segment data out of bounds");
                }

                address_space.write(
                    VirtAddr::new(vaddr + load_base),
                    &elf_data[offset..file_end],
                )?;

                if mem_size > file_size as u64 {
                    address_space.zero(
                        VirtAddr::new(vaddr + file_size as u64 + load_base),
                        (mem_size - file_size as u64) as usize,
                    )?;
                }
            },

            Type::Interp => {
                let bytes = elf_data[program_header.offset() as usize
                    ..(program_header.offset() + program_header.file_size())
                        as usize]
                    .iter()
                    .copied()
                    .take_while(|&b| b != 0)
                    .collect::<Vec<u8>>();

                let path = core::str::from_utf8(&bytes)
                    .map_err(|_| "invalid interp path")?;

                interp = Some(path.to_string());
            },

            _ => {},
        }
    }

    // find phdr
    let mut phdr_vaddr: Option<u64> = None;

    for header in elf.program_iter() {
        if matches!(header.get_type(), Ok(Type::Phdr)) {
            phdr_vaddr = Some(header.virtual_addr() + load_base);
            break;
        }
    }

    if phdr_vaddr.is_none() {
        let ph_offset = elf.header.pt2.ph_offset();

        for header in elf.program_iter() {
            if matches!(header.get_type(), Ok(Type::Load)) {
                continue;
            }

            let seg_start = header.offset();
            let seg_end = seg_start + header.file_size();
            if ph_offset >= seg_start && ph_offset < seg_end {
                phdr_vaddr = Some(
                    header.virtual_addr() + (ph_offset - seg_start) + load_base,
                );
                break;
            }
        }

        // worst case scenario
        if phdr_vaddr.is_none() {
            phdr_vaddr = Some(ph_offset + load_base);
        }
    }

    Ok(ELF64 {
        entry: elf.header.pt2.entry_point() + load_base,
        size: elf_data.len(),
        phdr: phdr_vaddr.unwrap(),
        phent: header.pt2.ph_entry_size() as u64,
        phnum: header.pt2.ph_count() as u64,
        interp,
        base: load_base,
        end: image_end,
    })
}

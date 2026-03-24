use x86_64::VirtAddr;
use x86_64::structures::paging::PageTableFlags;

use crate::arch;
use crate::system::mem::vmm::{self, AddressSpace};

const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ELF64Header {
    pub magic: [u8; 4],
    pub class: u8,
    pub endian: u8,
    pub version: u8,
    pub abi: u8,
    pub _pad: [u8; 8],
    pub elf_type: u16,
    pub machine: u16,
    pub version2: u32,
    pub entry: u64,
    pub phoff: u64,
    pub shoff: u64,
    pub flags: u32,
    pub ehsize: u16,
    pub phentsize: u16,
    pub phnum: u16,
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ELF64Phdr {
    pub seg_type: u32,
    pub flags: u32,
    pub offset: u64,
    pub vaddr: u64,
    pub paddr: u64,
    pub filesz: u64,
    pub memsz: u64,
    pub align: u64,
}

pub struct ELF64 {
    pub entry: u64,
    pub size: usize,
    pub phdr: u64,
    pub phent: u64,
    pub phnum: u64,
}

const PT_LOAD: u32 = 1;
const PT_PHDR: u32 = 6;
const PF_X: u32 = 1;
const PF_W: u32 = 2;
const PF_R: u32 = 4;

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

pub fn load(elf_data: &[u8]) -> Result<ELF64, &'static str> {
    if elf_data.len() < core::mem::size_of::<ELF64Header>() {
        return Err("invalid elf size");
    }

    let header = unsafe { &*(elf_data.as_ptr() as *const ELF64Header) };

    if header.magic != ELF_MAGIC {
        return Err("invalid elf magic");
    }

    if header.class != 2 {
        return Err("not an elf64");
    }

    if header.machine != 0x3E {
        return Err("not amda64");
    }

    if header.phentsize as usize != core::mem::size_of::<ELF64Phdr>() {
        return Err("invalid program header size");
    }

    let ph_offset = header.phoff as usize;
    let ph_size = header.phentsize as usize;
    let ph_num = header.phnum as usize;

    let ph_total = ph_size
        .checked_mul(ph_num)
        .ok_or("invalid program header table size")?;
    let ph_table_end = ph_offset
        .checked_add(ph_total)
        .ok_or("invalid program header table size")?;
    if ph_table_end > elf_data.len() {
        return Err("header out of bounds");
    }

    let mut phdr_vaddr = 0u64;

    for i in 0..ph_num {
        let ph_start = ph_offset + i * ph_size;
        if ph_start + ph_size > elf_data.len() {
            return Err("header out of bounds");
        }

        let ph =
            unsafe { &*(elf_data.as_ptr().add(ph_start) as *const ELF64Phdr) };

        if ph.seg_type == PT_PHDR {
            phdr_vaddr = ph.vaddr;
        }

        if ph.seg_type != PT_LOAD {
            continue;
        }

        if phdr_vaddr == 0 {
            let seg_file_start = ph.offset as usize;
            let seg_file_end =
                seg_file_start.saturating_add(ph.filesz as usize);
            if seg_file_start <= ph_offset && ph_table_end <= seg_file_end {
                phdr_vaddr = ph.vaddr + (header.phoff - ph.offset);
            }
        }

        let mut flags =
            PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;

        if (ph.flags & PF_W) != 0 {
            flags |= PageTableFlags::WRITABLE;
        }

        if (ph.flags & PF_X) == 0 {
            flags |= PageTableFlags::NO_EXECUTE;
        }

        let start_page = ph.vaddr & !0xFFF;
        let end_addr = ph.vaddr + ph.memsz;
        let end_page = (end_addr + 0xFFF) & !0xFFF;

        let mut addr = start_page;
        while addr < end_page {
            let page_addr = VirtAddr::new(addr);
            if !vmm::page_is_mapped(page_addr) {
                vmm::page_map_alloc(page_addr, flags)?;
            } else {
                let current = vmm::page_flags(page_addr)?;
                let merged = merge_page_flags(current, flags);
                vmm::page_update_flags(page_addr, merged)?;
            }
            addr += 4096;
        }

        let file_start = ph.offset as usize;
        let file_end = file_start + ph.filesz as usize;

        if file_end > elf_data.len() {
            return Err("segment data out of bounds");
        }

        unsafe {
            core::ptr::copy_nonoverlapping(
                elf_data.as_ptr().add(file_start),
                ph.vaddr as *mut u8,
                ph.filesz as usize,
            );

            if ph.memsz > ph.filesz {
                core::ptr::write_bytes(
                    (ph.vaddr + ph.filesz) as *mut u8,
                    0,
                    (ph.memsz - ph.filesz) as usize,
                );
            }
        }
    }

    Ok(ELF64 {
        entry: header.entry,
        size: elf_data.len(),
        phdr: phdr_vaddr,
        phent: header.phentsize as u64,
        phnum: header.phnum as u64,
    })
}

pub fn load_into(
    elf_data: &[u8],
    address_space: &AddressSpace,
) -> Result<ELF64, &'static str> {
    if elf_data.len() < core::mem::size_of::<ELF64Header>() {
        return Err("invalid elf size");
    }

    let header = unsafe { &*(elf_data.as_ptr() as *const ELF64Header) };

    if header.magic != ELF_MAGIC {
        return Err("invalid elf magic");
    }

    if header.class != 2 {
        return Err("not an elf64");
    }

    if header.machine != 0x3E {
        return Err("not amda64");
    }

    if header.phentsize as usize != core::mem::size_of::<ELF64Phdr>() {
        return Err("invalid program header size");
    }

    let ph_offset = header.phoff as usize;
    let ph_size = header.phentsize as usize;
    let ph_num = header.phnum as usize;

    let ph_total = ph_size
        .checked_mul(ph_num)
        .ok_or("invalid program header table size")?;
    let ph_table_end = ph_offset
        .checked_add(ph_total)
        .ok_or("invalid program header table size")?;
    if ph_table_end > elf_data.len() {
        return Err("header out of bounds");
    }

    let mut phdr_vaddr = 0;

    for i in 0..ph_num {
        let ph_start = ph_offset + i * ph_size;
        if ph_start + ph_size > elf_data.len() {
            return Err("header out of bounds");
        }

        let ph =
            unsafe { &*(elf_data.as_ptr().add(ph_start) as *const ELF64Phdr) };

        if ph.seg_type == PT_PHDR {
            phdr_vaddr = ph.vaddr;
        }

        if ph.seg_type != PT_LOAD {
            continue;
        }

        if phdr_vaddr == 0 {
            let seg_file_start = ph.offset as usize;
            let seg_file_end =
                seg_file_start.saturating_add(ph.filesz as usize);
            if seg_file_start <= ph_offset && ph_table_end <= seg_file_end {
                phdr_vaddr = ph.vaddr + (header.phoff - ph.offset);
            }
        }

        let mut flags =
            PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;

        if (ph.flags & PF_W) != 0 {
            flags |= PageTableFlags::WRITABLE;
        }

        if (ph.flags & PF_X) == 0 {
            flags |= PageTableFlags::NO_EXECUTE;
        }

        let start_page = ph.vaddr & !0xFFF;
        let end_addr = ph.vaddr + ph.memsz;
        let end_page = (end_addr + 0xFFF) & !0xFFF;

        let mut addr = start_page;
        while addr < end_page {
            let page_addr = VirtAddr::new(addr);
            if !address_space.is_mapped(page_addr) {
                address_space.map_page_alloc(page_addr, flags)?;
            } else {
                let current = address_space.page_flags(page_addr)?;
                let merged = merge_page_flags(current, flags);
                address_space.update_page_flags(page_addr, merged)?;
            }
            addr += arch::layout::PAGE_SIZE as u64;
        }

        let file_start = ph.offset as usize;
        let file_end = file_start + ph.filesz as usize;

        if file_end > elf_data.len() {
            return Err("segment data out of bounds");
        }

        address_space
            .write(VirtAddr::new(ph.vaddr), &elf_data[file_start..file_end])?;

        if ph.memsz > ph.filesz {
            address_space.zero(
                VirtAddr::new(ph.vaddr + ph.filesz),
                (ph.memsz - ph.filesz) as usize,
            )?;
        }
    }

    Ok(ELF64 {
        entry: header.entry,
        size: elf_data.len(),
        phdr: phdr_vaddr,
        phent: header.phentsize as u64,
        phnum: header.phnum as u64,
    })
}

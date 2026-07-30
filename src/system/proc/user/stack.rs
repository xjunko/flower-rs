use alloc::vec::Vec;

use flower_mono::auxv::AuxType;
use x86_64::VirtAddr;
use x86_64::structures::paging::PageTableFlags;

use crate::arch::x86_64::layout::{
    USER_STACK_INITIAL_SLACK, USER_STACK_PAGES, USER_STACK_TOP_PAGE,
};
use crate::memory::vmm::AddressSpace;
use crate::system::elf::{self, ELFLoadType};
use crate::system::vfs::VFSFilelike;
use crate::{arch, system};

pub struct UserImageInfo {
    pub address_space: AddressSpace,
    pub entry: u64,
    pub stack_ptr: u64,
    pub stack_bottom: u64,
    pub heap_start: u64,
    pub heap_max: u64,
}

const PAGE_SIZE: u64 = arch::x86_64::layout::PAGE_SIZE as u64;
struct StackBuilder<'a> {
    pub stack_pointer: u64,
    stack_bottom: u64,
    adress_space: &'a AddressSpace,
}

impl<'a> StackBuilder<'a> {
    fn new(
        stack_top: u64,
        stack_bottom: u64,
        address_space: &'a AddressSpace,
    ) -> Self {
        Self {
            stack_pointer: stack_top,
            stack_bottom,
            adress_space: address_space,
        }
    }

    fn push(&mut self, value: u64) -> Result<u64, &'static str> {
        self.stack_pointer =
            self.stack_pointer.checked_sub(8).ok_or("user stack overflow")?;
        if self.stack_pointer < self.stack_bottom {
            return Err("user stack overflow");
        }
        self.adress_space
            .write(VirtAddr::new(self.stack_pointer), &value.to_ne_bytes())?;
        Ok(self.stack_pointer)
    }

    fn push_bytes(&mut self, data: &[u8]) -> Result<u64, &'static str> {
        let len = data.len() as u64;
        self.stack_pointer =
            self.stack_pointer.checked_sub(len).ok_or("user stack overflow")?;
        if self.stack_pointer < self.stack_bottom {
            return Err("user stack overflow");
        }
        self.adress_space.write(VirtAddr::new(self.stack_pointer), data)?;
        Ok(self.stack_pointer)
    }
}

// auxv
impl<'a> StackBuilder<'a> {
    fn push_auxv_from_elf(
        &mut self,
        program: &elf::ELF64,
    ) -> Result<(), &'static str> {
        // the reason we're building on the reversed order
        // is because the stack grows downwards
        self.push_auxv(AuxType::Null, 0)?;
        self.push_auxv(AuxType::Entry, program.entry)?;
        self.push_auxv(AuxType::PageSize, PAGE_SIZE)?;
        self.push_auxv(AuxType::Phnum, program.phnum)?;
        self.push_auxv(AuxType::Phent, program.phent)?;
        if program.phdr != 0 {
            self.push_auxv(AuxType::Phdr, program.phdr)?;
        }
        Ok(())
    }

    pub fn push_auxv(
        &mut self,
        aux_type: AuxType,
        value: u64,
    ) -> Result<u64, &'static str> {
        self.push(value)?;
        self.push(aux_type as u64)?;
        Ok(self.stack_pointer)
    }
}

// stack final
impl<'a> StackBuilder<'a> {
    fn finalize(self) -> u64 {
        if !self.stack_pointer.is_multiple_of(16) {
            panic!("user stack not aligned to 16 bytes");
        }

        if self.stack_pointer < self.stack_bottom {
            panic!("user stack underflow at finalize");
        }

        self.stack_pointer
    }
}

pub fn build_user_image(
    elf_data: &[u8],
    argv: &[&str],
) -> Result<UserImageInfo, &'static str> {
    let address_space = AddressSpace::new()?;
    let loaded = elf::load_into(elf_data, &address_space, ELFLoadType::Main)?;

    let mut entry = loaded.entry;
    let mut base = loaded.base;

    // NOTE: dynamic linking, still a bit hacky, but it works
    if let Some(interp_path) = &loaded.interp {
        if let Ok(interp_file) = system::vfs::open(interp_path, 0) {
            match interp_file {
                VFSFilelike::File(f) => {
                    let metadata =
                        f.metadata().expect("failed to get interp metadata.");
                    let mut buffer = alloc::vec![0u8; metadata.size];
                    f.read(&mut buffer).expect("failed to read interp");
                    let interp_loaded = elf::load_into(
                        &buffer,
                        &address_space,
                        ELFLoadType::Interpreter,
                    )
                    .expect("failed to load elf");
                    entry = interp_loaded.entry;
                    base = interp_loaded.base;
                },
                _ => return Err("interpreter is not a regular file"),
            };
        } else {
            return Err("failed to open interpreter");
        }
    }

    if !address_space.is_mapped(VirtAddr::new(entry & !0xFFF)) {
        return Err("entry point is not mapped");
    }

    let flags = PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::USER_ACCESSIBLE;

    // Allocate initial heap page (will grow on demand)
    let mut user_heap = loaded.end;
    user_heap = (user_heap + PAGE_SIZE - 1) & !0xFFF;
    address_space
        .map_page_alloc(VirtAddr::new(user_heap), flags)
        .expect("failed to allocate initial heap");
    let heap_max = user_heap + (512 * PAGE_SIZE); // Allow heap to grow up to 512 pages (2MB)

    // Allocate only the initial stack page (will grow on demand)
    let stack_top_page = USER_STACK_TOP_PAGE;
    address_space
        .map_page_alloc(VirtAddr::new(stack_top_page), flags)
        .expect("failed to allocate initial stack");

    let stack_low = USER_STACK_TOP_PAGE - (USER_STACK_PAGES * PAGE_SIZE);
    let user_stack_top =
        (USER_STACK_TOP_PAGE + PAGE_SIZE - USER_STACK_INITIAL_SLACK) & !0xF;
    debug_assert!(
        user_stack_top <= USER_STACK_TOP_PAGE && user_stack_top >= stack_low
    );

    let user_stack = {
        let mut stack_builder =
            StackBuilder::new(user_stack_top, stack_low, &address_space);

        // argv
        let mut argv_ptrs: Vec<u64> = Vec::with_capacity(argv.len());
        for arg in argv.iter().rev() {
            let mut bytes = Vec::from(arg.as_bytes());
            bytes.push(0);

            let arg_ptr = stack_builder.push_bytes(&bytes)?;
            argv_ptrs.push(arg_ptr);
        }
        argv_ptrs.reverse();

        // align
        stack_builder.stack_pointer &= !0xF;
        if stack_builder.stack_pointer < stack_low {
            return Err("user stack overflow");
        }

        // note: this got my ass so hard
        // had to use clankers for this
        // need (3 + argc) to be even -> need argc to be odd.
        // if argc is even, push one padding word now to compensate.
        if argv_ptrs.len().is_multiple_of(2) {
            stack_builder.push(0)?;
        }

        // auxv
        stack_builder.push_auxv_from_elf(&loaded)?;

        if base != 0 {
            stack_builder.push_auxv(AuxType::Base, base)?;
        }

        // envp
        stack_builder.push(0)?;
        stack_builder.push(0)?;

        // argv pointers
        for ptr in argv_ptrs.iter().rev() {
            stack_builder.push(*ptr)?;
        }

        // argc
        stack_builder.push(argv_ptrs.len() as u64)?;

        // finalize stack
        stack_builder.finalize()
    };

    log::debug!("User stack built at {:#x}", user_stack);

    Ok(UserImageInfo {
        address_space,
        entry,
        stack_ptr: user_stack,
        stack_bottom: stack_low,
        heap_start: user_heap,
        heap_max,
    })
}

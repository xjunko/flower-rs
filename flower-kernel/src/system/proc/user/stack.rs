use alloc::vec::Vec;

use flower_mono::auxv::AuxType;
use x86_64::VirtAddr;
use x86_64::structures::paging::PageTableFlags;

use crate::arch;
use crate::arch::layout::{
    USER_STACK_INITIAL_SLACK, USER_STACK_PAGES, USER_STACK_TOP_PAGE,
};
use crate::system::elf;
use crate::system::mem::vmm::AddressSpace;

const PAGE_SIZE: u64 = arch::layout::PAGE_SIZE as u64;
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
            stack_bottom: stack_bottom,
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
        self.push_auxv(AuxType::PageSize, PAGE_SIZE as u64)?;
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
        let _ = self.push(value);
        let _ = self.push(aux_type as u64);
        Ok(self.stack_pointer)
    }
}

// stack final
impl<'a> StackBuilder<'a> {
    fn finalize(self) -> u64 {
        if self.stack_pointer % 16 != 0 {
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
) -> Result<(AddressSpace, u64, u64, u64), &'static str> {
    let address_space = AddressSpace::new()?;
    let loaded = elf::load_into(elf_data, &address_space)?;

    if !address_space.is_mapped(VirtAddr::new(loaded.entry & !0xFFF)) {
        return Err("entry point is not mapped");
    }

    let flags = PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::USER_ACCESSIBLE;

    let mut user_heap = loaded.entry + loaded.size as u64;
    user_heap = (user_heap + PAGE_SIZE - 1) & !0xFFF;
    address_space.map_page_alloc(VirtAddr::new(user_heap), flags)?;
    user_heap += PAGE_SIZE;

    for i in 0..USER_STACK_PAGES {
        let page_addr = USER_STACK_TOP_PAGE - (i * PAGE_SIZE);
        address_space.map_page_alloc(VirtAddr::new(page_addr), flags)?;
    }

    let stack_low =
        USER_STACK_TOP_PAGE + PAGE_SIZE - (USER_STACK_PAGES * PAGE_SIZE);
    let user_stack_top =
        (USER_STACK_TOP_PAGE + PAGE_SIZE - USER_STACK_INITIAL_SLACK) & !0xF;
    debug_assert!(
        user_stack_top >= stack_low
            && user_stack_top < USER_STACK_TOP_PAGE + PAGE_SIZE
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

        // auxv
        stack_builder.push_auxv_from_elf(&loaded)?;

        // envp
        stack_builder.push(0)?;
        stack_builder.push(0)?;

        // argv pointers
        for ptr in argv_ptrs.iter().rev() {
            stack_builder.push(*ptr)?;
        }

        // argc
        stack_builder.push(argv_ptrs.len() as u64)?;

        // align
        stack_builder.stack_pointer &= !0xF;
        if stack_builder.stack_pointer < stack_low {
            return Err("user stack overflow");
        }

        // finalize stack
        stack_builder.finalize()
    };
    log::info!("User stack built at {:#x}", user_stack);

    Ok((address_space, loaded.entry, user_stack, user_heap))
}

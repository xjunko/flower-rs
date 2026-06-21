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
    stack_top: VirtAddr,
    stack_bottom: VirtAddr,
    adress_space: &'a AddressSpace,
}

impl<'a> StackBuilder<'a> {
    fn new(
        stack_top: u64,
        stack_bottom: u64,
        address_space: &'a AddressSpace,
    ) -> Self {
        Self {
            stack_top: VirtAddr::new(stack_top),
            stack_bottom: VirtAddr::new(stack_bottom),
            adress_space: address_space,
        }
    }

    fn push(&mut self, value: u64) -> u64 {
        {
            let new_top = self
                .stack_top
                .as_u64()
                .checked_sub(8u64)
                .expect("user stack underflow");

            if new_top < self.stack_bottom.as_u64() {
                panic!("user stack overflow");
            }

            self.stack_top = VirtAddr::new(new_top);
        }

        self.adress_space.write(self.stack_top, &value.to_ne_bytes()).unwrap();
        self.stack_top.as_u64()
    }

    fn push_bytes(&mut self, data: &[u8]) -> u64 {
        {
            let len = data.len() as u64;
            let new_top = self
                .stack_top
                .as_u64()
                .checked_sub(len)
                .expect("user stack underflow");

            if new_top < self.stack_bottom.as_u64() {
                panic!("user stack overflow");
            }

            self.stack_top = VirtAddr::new(new_top);
        }
        self.adress_space.write(self.stack_top, data).unwrap();
        self.stack_top.as_u64()
    }
}

// auxv
impl<'a> StackBuilder<'a> {
    fn push_auxv_from_elf(&mut self, program: &elf::ELF64) {
        // the reason we're building on the reversed order
        // is because the stack grows downwards
        self.push_auxv(AuxType::Null, 0);
        self.push_auxv(AuxType::Entry, program.entry);
        self.push_auxv(AuxType::PageSize, PAGE_SIZE as u64);
        self.push_auxv(AuxType::Phnum, program.phnum);
        self.push_auxv(AuxType::Phent, program.phent);
        if program.phdr != 0 {
            self.push_auxv(AuxType::Phdr, program.phdr);
        }
    }

    pub fn push_auxv(&mut self, aux_type: AuxType, value: u64) {
        self.push(value);
        self.push(aux_type as u64);
    }
}

// stack final
impl<'a> StackBuilder<'a> {
    fn align_down(&mut self, align: u64) {
        let aligned = self.stack_top.as_u64() & !(align - 1);

        if VirtAddr::new(aligned) < self.stack_bottom {
            panic!("user stack overflow");
        }

        self.stack_top = VirtAddr::new(aligned);
    }

    fn finalize(self) -> u64 {
        if self.stack_top.as_u64() % 16 != 0 {
            panic!("user stack not aligned to 16 bytes");
        }

        if self.stack_top < self.stack_bottom {
            panic!("user stack underflow at finalize");
        }

        self.stack_top.as_u64()
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

            let ptr = stack_builder.push_bytes(&bytes);
            argv_ptrs.push(ptr);
        }
        argv_ptrs.reverse();

        // align
        stack_builder.align_down(16);

        // auxv
        stack_builder.push_auxv_from_elf(&loaded);

        // envp
        stack_builder.push(0);
        stack_builder.push(0);

        // argv pointers
        for ptr in argv_ptrs.iter().rev() {
            stack_builder.push(*ptr);
        }

        // argc
        stack_builder.push(argv.len() as u64);

        // finalize stack
        stack_builder.finalize()
    };

    Ok((address_space, loaded.entry, user_stack, user_heap))
}

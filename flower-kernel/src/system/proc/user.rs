use x86_64::VirtAddr;
use x86_64::structures::paging::PageTableFlags;

use crate::system::elf;
use crate::system::mem::vmm::AddressSpace;
use crate::system::proc::{
    PAGE_SIZE, USER_STACK_INITIAL_SLACK, USER_STACK_PAGES, USER_STACK_TOP_PAGE,
    auxv,
};

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

    let user_stack = auxv::build_initial_user_stack(
        argv,
        &address_space,
        stack_low,
        user_stack_top,
        &loaded,
    )?;

    Ok((address_space, loaded.entry, user_stack, user_heap))
}

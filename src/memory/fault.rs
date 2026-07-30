use x86_64::VirtAddr;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};
use x86_64::structures::paging::PageTableFlags;

use crate::arch::x86_64::idt::print_stack_frame;
use crate::{println, system};

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let fault_addr = match Cr2::read() {
        Ok(addr) => addr.as_u64(),
        Err(addr_err) => {
            log::error!(
                "page fault triggered, but CR2 is invalid: {:?}",
                addr_err
            );
            system::proc::exit(1);
            return;
        },
    };

    // we might be able to handle this
    if let Some(current) = system::proc::current() {
        let mut proc = current.lock();

        // okay, it's from the user, probably fine.
        if proc.level == system::proc::ProcessLevel::RING3 {
            let stack_bottom = proc.user_stack_bottom();
            let stack_top = proc.user_stack_top();
            let heap_top = proc.user_heap_top();
            let heap_max = proc.user_heap_max();

            // inside the stack region?
            if fault_addr >= stack_bottom && fault_addr <= stack_top {
                // inside the region, just allocate...
                if let Some(address_space) = proc.address_space.as_ref() {
                    let page_addr = VirtAddr::new(fault_addr & !0xFFF);
                    let flags = PageTableFlags::PRESENT
                        | PageTableFlags::WRITABLE
                        | PageTableFlags::USER_ACCESSIBLE;

                    log::debug!(
                        "stack fault: addr={:#x} page={:#x} already_mapped={}",
                        fault_addr,
                        page_addr.as_u64(),
                        address_space.is_mapped(page_addr)
                    );

                    match address_space.map_page_alloc(page_addr, flags) {
                        Ok(_) => {
                            if page_addr.as_u64() < stack_bottom {
                                proc.set_user_stack_bounds(
                                    page_addr.as_u64(),
                                    stack_top,
                                );
                            }
                            log::debug!(
                                "allocated stack page at {:#x}",
                                page_addr.as_u64()
                            );
                            return;
                        },
                        Err(e) => {
                            log::error!(
                                "stack demand-page alloc FAILED at {:#x}: {}",
                                page_addr.as_u64(),
                                e
                            );
                        },
                    }
                }
            }

            // inside the heap region?
            if fault_addr >= heap_top && fault_addr < heap_max {
                // inside the region, just allocate...
                if let Some(address_space) = proc.address_space.as_ref() {
                    let page_addr = VirtAddr::new(fault_addr & !0xFFF);
                    let flags = PageTableFlags::PRESENT
                        | PageTableFlags::WRITABLE
                        | PageTableFlags::USER_ACCESSIBLE;

                    if address_space.map_page_alloc(page_addr, flags).is_ok() {
                        // extend heap, if necessary
                        if page_addr.as_u64() + 0x1000 > heap_top {
                            proc.set_user_heap_bounds(
                                page_addr.as_u64() + 0x1000,
                                heap_max,
                            );
                        }
                        log::debug!(
                            "allocated heap page at {:#x}",
                            page_addr.as_u64()
                        );
                        return;
                    }
                }
            }
        }
    }

    // shit hits the fan, panic out.
    log::error!("page fault triggered, in process: {}", system::proc::name());
    println!("CR2:        {:#x}", fault_addr);
    println!("error code: {:#x}", error_code);
    print_stack_frame(stack_frame);

    system::proc::exit(1);
}

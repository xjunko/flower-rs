use alloc::string::{String, ToString};

use x86_64::PhysAddr;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::PhysFrame;

use crate::system::proc::ProcessLevel;
use crate::system::syscalls::SyscallFrame;
use crate::system::{self, vfs};

fn process_name_from_path(path: &str) -> String {
    path.rsplit('/').find(|part| !part.is_empty()).unwrap_or(path).to_string()
}

/// replaces the current process image with a new one from the given path.
pub fn execve(
    path: &str,
    frame: &mut SyscallFrame,
) -> Result<(), &'static str> {
    let elf_data = vfs::__read(path)?;
    let name = process_name_from_path(path);
    let (address_space, user_entry, user_stack, user_heap) =
        system::proc::build_user_image(&name, &elf_data)?;

    let new_cr3 = address_space.cr3();
    let (current_frame, current_flags) = Cr3::read();
    if current_frame.start_address().as_u64() != new_cr3 {
        let new_frame = PhysFrame::containing_address(PhysAddr::new(new_cr3));
        unsafe {
            Cr3::write(new_frame, current_flags);
        }
    }

    let current = system::proc::current().ok_or("no current process")?;
    let mut proc = current.lock();

    if proc.level != ProcessLevel::RING3 {
        return Err("execve is only supported for user processes");
    }

    let old_address_space = proc.address_space.take();

    proc.name = name;
    proc.address_space = Some(address_space);
    proc.cr3 = new_cr3;
    proc.user_entry = user_entry;
    proc.user_stack = user_stack;
    proc.user_heap = user_heap;
    proc.user_heap_position = user_heap;
    proc._fsbase = 0;

    log::trace!("execve: address space switched, preparing to switch stacks");
    unsafe {
        proc.switch_stack();
    }
    log::trace!(
        "execve: stacks switched, dropping old address space and writing fsbase"
    );

    drop(old_address_space);

    frame.rip = user_entry;
    frame.rsp = user_stack;
    frame.rax = 0;

    Ok(())
}

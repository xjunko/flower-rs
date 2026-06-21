use crate::system::proc::user::stack::build_user_image;
use crate::system::proc::{Process, SCHEDULER};

mod auxv;
mod execve;
mod fork;
mod stack;
mod waitpid;

pub use execve::execve;
pub use fork::fork;
pub use waitpid::waitpid;

/// spawns an elf process with the given name and elf bytes.
pub fn spawn_elf(name: &str, elf_data: &[u8]) -> Result<u64, &'static str> {
    let argv = [name];
    let (address_space, user_entry, user_stack, user_heap) =
        build_user_image(elf_data, &argv)?;

    let proc = Process::new_user(
        name,
        address_space,
        user_entry,
        user_stack,
        user_heap,
    );
    let proc_id = proc.id;
    log::trace!(
        "created process {} with entry point {:#x}",
        proc.name,
        user_entry
    );

    if let Some(sched) = SCHEDULER.lock().as_mut() {
        sched.add(proc);
    }

    Ok(proc_id)
}

use crate::system;
use crate::system::proc::{Process, ProcessLevel};
use crate::system::syscalls::SyscallFrame;

/// creates a new process by copying the current one, and returns the new pid.
pub fn fork(frame: &SyscallFrame) -> Result<u64, &'static str> {
    let current = system::proc::current().ok_or("no current process")?;
    let parent = current.lock();

    if parent.level != ProcessLevel::RING3 {
        return Err("fork is only supported for user processes");
    }

    let parent_as = parent
        .address_space
        .as_ref()
        .ok_or("user process has no address space")?;
    let child_as = parent_as.clone_user()?;

    let child = Process::new_forked(&parent, child_as, frame);
    let child_id = child.id;
    drop(parent);

    let mut sched_guard = system::proc::SCHEDULER.lock();
    let sched = sched_guard.as_mut().ok_or("scheduler not initialized")?;
    sched.add(child);

    Ok(child_id)
}

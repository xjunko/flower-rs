use x86_64::instructions::interrupts;

use crate::system::proc::{ProcessState, schedule};
use crate::system::{self};

/// exits the current process.
pub fn exit() {
    interrupts::without_interrupts(|| {
        system::syscalls::write_cpu_context();
        if let Some(sched) = system::proc::SCHEDULER.lock().as_mut() {
            if let Some(proc) = sched.current() {
                let mut proc = proc.lock();
                proc.state = ProcessState::Dead;
            } else {
                panic!("trying to exit while no process is running!");
            }
        } else {
            panic!("trying to exit while not initialized!");
        }
    });
    schedule();
    unreachable!();
}

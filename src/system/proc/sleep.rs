use x86_64::instructions::interrupts;

use crate::arch;
use crate::system::proc::{ProcessState, SCHEDULER, schedule};
use crate::system::{self};

/// sleeps the current process for the given number of milliseconds.
pub fn sleep(millis: u64) {
    let wake_at = arch::x86_64::ticks() + millis;

    interrupts::without_interrupts(|| {
        system::syscalls::write_cpu_context();
        if let Some(sched) = SCHEDULER.lock().as_mut() {
            if let Some(proc) = sched.current() {
                let mut proc = proc.lock();
                proc.state = ProcessState::Sleeping(wake_at);
            } else {
                panic!("trying to sleep while no process is running!");
            }
        } else {
            panic!("trying to sleep while not initialized!");
        }
    });
    schedule();
}

use x86_64::instructions::interrupts;

use crate::system;

pub fn waitpid(pid: u64) -> Result<u64, &'static str> {
    loop {
        let result = interrupts::without_interrupts(|| {
            let mut guard = system::proc::SCHEDULER.lock();
            let sched = guard.as_mut().ok_or("scheduler not initialized")?;

            let current = sched.current().ok_or("no current process")?;
            let current_id = current.lock().id;

            let child_idx = sched
                .processes
                .iter()
                .position(|proc| {
                    let proc = proc.lock();
                    proc.id == pid && proc.parent_id == Some(current_id)
                })
                .ok_or("no child process")?;

            let status = {
                let child = sched.processes[child_idx].lock();
                if child.state != system::proc::ProcessState::Zombie {
                    return Ok(None);
                }

                child.exit_status.unwrap_or(0)
            };

            sched.processes.remove(child_idx);
            if child_idx < sched.current {
                sched.current -= 1;
            }

            Ok(Some(status))
        });

        match result {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => system::proc::schedule(),
            Err(e) => return Err(e),
        }
    }
}

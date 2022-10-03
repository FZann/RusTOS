use crate::kernel::BooleanVector;
use crate::kernel::processes::{Process, ProcessState};
use crate::kernel::scheduler::{Scheduler, SCHEDULER};


pub struct Semaphore {
    locked: BooleanVector,
}

impl Semaphore {
    pub fn new() -> Self {
        Semaphore {
            locked: BooleanVector::new(),
        }
    }

    pub fn wait(&self) {
        let sched = unsafe { &mut SCHEDULER };
            if let Some(pcb_ptr) = sched.process_running {
                let pcb = unsafe {&*pcb_ptr};
                self.locked.set(pcb.prio());
                pcb.set_state(ProcessState::Stopped);
                sched.run_next();
        }
    }

    pub fn release(&self) {
        let sched = unsafe { &mut SCHEDULER };
        if let Ok(prio) = self.locked.find_first_set() {
            self.locked.clear(prio as u8);
            sched.process_idle(prio as u8);
            sched.run_next();
        }
    }
}

use crate::kernel::processes::{Process, ProcessState};
use crate::kernel::scheduler::{Scheduler, SCHEDULER};
use crate::kernel::{BitVec, BitVector};

pub struct Semaphore {
    locked: BitVec,
}

impl Semaphore {
    pub fn new() -> Self {
        Semaphore { locked: 0 }
    }

    pub fn wait(&self) {
        let sched = unsafe { &mut SCHEDULER };
        if let Some(pcb) = sched.running {
            //self.locked.set(pcb.prio() as usize);
            pcb.set_state(ProcessState::Stopped);
            sched.schedule_next();
        }
    }

    pub fn release(&self) {
        let sched = unsafe { &mut SCHEDULER };
        if let Ok(prio) = self.locked.first_set() {
            //self.locked.clear(prio as usize);
            sched.process_idle(prio);
            sched.schedule_next();
        }
    }
}

use crate::kernel::{BitVec, BitVector};
use crate::kernel::processes::{Process, ProcessState};
use crate::kernel::scheduler::{Scheduler, SCHEDULER};


pub struct Semaphore {
    locked: BitVec,
}

impl Semaphore {
    pub fn new() -> Self {
        Semaphore {
            locked: 0,
        }
    }

    pub fn wait(&self) {
        let sched = unsafe { &mut SCHEDULER };
            if let Some(pcb_ptr) = sched.process_running {
                let pcb = unsafe {&*pcb_ptr};
                //self.locked.set(pcb.prio() as usize);
                pcb.set_state(ProcessState::Stopped);
                sched.run_next();
        }
    }

    pub fn release(&self) {
        let sched = unsafe { &mut SCHEDULER };
        if let Ok(prio) = self.locked.first_set() {
            //self.locked.clear(prio as usize);
            sched.process_idle(prio as u8);
            sched.run_next();
        }
    }
}

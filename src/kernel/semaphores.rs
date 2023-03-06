use crate::kernel::processes::Process;
use crate::kernel::scheduler::{Scheduler, SCHEDULER};
use crate::kernel::BitVec;

pub struct VecSemaphore {
    locked: BitVec,
}

impl VecSemaphore {
    pub const fn new() -> Self {
        Self {
            locked: BitVec::new(),
        }
    }

    pub fn wait(&self) {}

    pub fn release(&self) {}
}

pub struct Semaphore<'p> {
    locked: Option<&'p dyn Process>,
}

impl<'p> Semaphore<'p> {
    pub const fn new() -> Self {
        Self { locked: None }
    }

    pub fn wait(&mut self) {
        if self.locked.is_some() {
            return;
        }
        
        let sched = unsafe { &mut SCHEDULER };
        self.locked = sched.running;
        sched.running_stop();
    }

    pub fn release(&mut self) {
        if self.locked.is_none() {
            return;
        }

        let sched = unsafe { &mut SCHEDULER };
        sched.process_idle(self.locked.unwrap().prio());
        self.locked = None;
    }
}

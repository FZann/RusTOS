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

    pub fn wait(&mut self) {
        let sched = unsafe { &mut SCHEDULER };
        self.locked.set(sched.running_id());
        sched.running_stop();
    }

    pub fn release(&mut self) {
        if let Ok(id) = self.locked.first_set() {
            let sched = unsafe { &mut SCHEDULER };
            self.locked.clear(id);
            sched.process_idle(id);
        }
    }
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
        if let Some(locked) = self.locked {
            let sched = unsafe { &mut SCHEDULER };
            let prio = locked.prio();
            self.locked = None;
            sched.process_idle(prio);
        }
    }
}

use crate::kernel::processes::Process;
use crate::kernel::scheduler::{SCHEDULER, Scheduler};
use crate::kernel::BitVec;

use super::Syncable;


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
        self.locked.set(SCHEDULER.running_id());
        SCHEDULER.cs(|s| s.running_stop());
    }

    pub fn release(&mut self) {
        if let Ok(id) = self.locked.first_set() {
            self.locked.clear(id);
            SCHEDULER.cs(|s| s.process_idle(id));
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

    self.locked = SCHEDULER.running;
    SCHEDULER.cs(|s| s.running_stop());
    }

    pub fn release(&mut self) {
        if let Some(locked) = self.locked {
            let prio = locked.prio();
            self.locked = None;
            SCHEDULER.cs(|s| s.process_idle(prio));
        }
    }
}

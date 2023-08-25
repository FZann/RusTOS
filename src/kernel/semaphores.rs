use crate::kernel::processes::Process;
use crate::kernel::scheduler::{SCHEDULER, Scheduler};
use crate::kernel::BitVec;

use super::CriticalSection;

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
        let s = SCHEDULER.get_access(&CriticalSection::activate());
        self.locked.set(s.running_id());
        s.running_stop();
    }

    pub fn release(&mut self) {
        if let Ok(id) = self.locked.first_set() {
            self.locked.clear(id);
            let s = SCHEDULER.get_access(&CriticalSection::activate());
            s.process_idle(id);
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

        let s = SCHEDULER.get_access(&CriticalSection::activate());
        self.locked = s.running;
        s.running_stop();
    }

    pub fn release(&mut self) {
        if let Some(locked) = self.locked {
            let prio = locked.prio();
            self.locked = None;
            let s = SCHEDULER.get_access(&CriticalSection::activate());
            s.process_idle(prio);
        }
    }
}

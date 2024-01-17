use crate::kernel::processes::Process;
use crate::kernel::scheduler::{SCHEDULER, Scheduler};
use crate::kernel::BitVec;
use crate::kernel::CriticalSection;

pub struct VecSemaphore {
    locked: BitVec,
}

impl VecSemaphore {
    pub const fn new() -> Self {
        Self {
            locked: BitVec::new(),
        }
    }

    pub fn wait(&mut self, _cs: &CriticalSection) {
        unsafe {
            self.locked.set(SCHEDULER.running_id());
            SCHEDULER.running_stop();
        }
    }

    pub fn release(&mut self, _cs: &CriticalSection) {
        if let Ok(id) = self.locked.first_set() {
            unsafe {
                self.locked.clear(id);
                SCHEDULER.process_idle(id);
            }
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

    pub fn wait(&mut self, _cs: &CriticalSection) {
        if self.locked.is_some() {
            return;
        }
        unsafe {
            self.locked = SCHEDULER.running;
            SCHEDULER.running_stop();
        }
    }

    pub fn release(&mut self, _cs: &CriticalSection) {
        if let Some(locked) = self.locked {
            let prio = locked.prio();
            self.locked = None;
            unsafe {
                SCHEDULER.process_idle(prio);
            }
        }
    }
}

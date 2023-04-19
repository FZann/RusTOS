use crate::kernel::processes::Process;
use crate::kernel::scheduler::{SCHEDULER, Scheduler};
use crate::kernel::{BitVec, Syncable};


pub struct VecSemaphore {
    locked: BitVec,
}

impl Syncable for VecSemaphore {}

impl VecSemaphore {
    pub const fn new() -> Self {
        Self {
            locked: BitVec::new(),
        }
    }

    pub fn wait(&mut self) {
        unsafe {
            self.locked.set(SCHEDULER.running_id());
            SCHEDULER.running_stop();
        }
    }

    pub fn release(&mut self) {
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

unsafe impl<'p> Sync for Semaphore<'p> {}
impl<'p> Syncable for Semaphore<'p> {}

impl<'p> Semaphore<'p> {
    pub const fn new() -> Self {
        Self { locked: None }
    }

    pub fn wait(&mut self) {
        if self.locked.is_some() {
            return;
        }

    unsafe {
        self.locked = SCHEDULER.running;
        SCHEDULER.running_stop();
    }
    }

    pub fn release(&mut self) {
        if let Some(locked) = self.locked {
            unsafe {
                let prio = locked.prio();
                self.locked = None;
                SCHEDULER.process_idle(prio);
            }
        }
    }
}

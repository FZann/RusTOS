use crate::kernel::processes::Process;
use crate::kernel::scheduler::{Scheduler, SCHEDULER};
use crate::kernel::BitVec;

use super::{SyncShare, Syncable};


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

    pub const fn new_syncable() -> SyncShare<Self> {
        SyncShare::new(Self::new())
    }

    pub fn wait(&mut self) {
        SCHEDULER.cs(|sched| {
            self.locked.set(sched.running_id());
            sched.running_stop();
        });
    }

    pub fn release(&mut self) {
        if let Ok(id) = self.locked.first_set() {
            SCHEDULER.cs(|sched| {
                self.locked.clear(id);
                sched.process_idle(id);
            });
        }
    }
}

pub struct Semaphore<'p> {
    locked: Option<&'p dyn Process>,
}

impl<'p> Syncable for Semaphore<'p> {}

impl<'p> Semaphore<'p> {
    pub const fn new() -> Self {
        Self { locked: None }
    }

    pub const fn new_syncable() -> SyncShare<Self> {
        SyncShare::new(Self::new())
    }

    pub fn wait(&mut self) {
        if self.locked.is_some() {
            return;
        }

        SCHEDULER.cs(|sched| {
            self.locked = sched.running;
            sched.running_stop();
        });
    }

    pub fn release(&mut self) {
        if let Some(locked) = self.locked {
            SCHEDULER.cs(|sched| {
                let prio = locked.prio();
                self.locked = None;
                sched.process_idle(prio);
            });
        }
    }
}

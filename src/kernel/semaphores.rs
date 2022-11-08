use core::cell::Cell;

use crate::kernel::scheduler::{Scheduler, SCHEDULER};
use crate::kernel::BitVec;

pub struct Semaphore {
    locked: Cell<BitVec>,
}

impl Semaphore {
    pub const fn new() -> Self {
        Semaphore {
            locked: Cell::new(BitVec::new()),
        }
    }

    pub fn wait(&self) {
        let sched = unsafe { &mut SCHEDULER };
        let prio = sched.running.unwrap().prio() as usize;

        let mut locked = self.locked.get();
        locked.set(prio);
        self.locked.set(locked);

        sched.process_stop(prio);
        sched.schedule_next();
    }

    pub fn release(&self) {
        let sched = unsafe { &mut SCHEDULER };
        let mut locked = self.locked.get();

        if let Ok(prio) = locked.first_set() {
            locked.clear(prio);
            self.locked.set(locked);

            sched.process_idle(prio);
            sched.schedule_next();
        }
    }
}

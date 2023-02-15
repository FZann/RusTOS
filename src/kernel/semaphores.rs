use core::cell::Cell;
use crate::kernel::scheduler::{Scheduler, SCHEDULER};
use super::processes::Process;


pub struct BinSem<'p> {
    locked: Option<&'p dyn Process>,
}

impl<'p> BinSem<'p> {
    pub const fn new() -> Self {
        Self {
            locked: None,
        }
    }

    pub fn wait(&mut self) {
        if self.locked.is_some() {
            panic!("Doppio lock!");
        }
        let sched = unsafe { &mut SCHEDULER };
        self.locked = sched.running;
        sched.process_stop(sched.running.unwrap().prio() as usize);
        sched.schedule_next();
    }

    pub fn release(&mut self) {
        if self.locked.is_none() {
            panic!("Release a vuoto!");
        }

        let sched = unsafe { &mut SCHEDULER };
        sched.process_idle(self.locked.unwrap().prio() as usize);
        self.locked = None;
        sched.schedule_next();
    }

    
}
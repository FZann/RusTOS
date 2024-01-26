use core::cell::Cell;

use crate::kernel::processes::Process;
use crate::kernel::scheduler::{SCHEDULER, Scheduler};
use crate::kernel::BitVec;

use super::{SysCallType, SystemCall, CriticalSection};

pub struct VecSemaphore {
    locked: Cell<BitVec>,
}

impl VecSemaphore {
    pub const fn new() -> Self {
        Self {
            locked: Cell::new(BitVec::new()),
        }
    }

    pub fn wait(&self) {
        let cs = CriticalSection::activate();
        let mut locked = self.locked.get();
        let id = SCHEDULER.get(&cs).running_id();
        locked.set(id);
        self.locked.set(locked);
        cs.deactivate();
        SystemCall(SysCallType::ProcessStop(id));
    }

    pub fn release(&self) {
        let cs = CriticalSection::activate();
        let mut locked = self.locked.get();

        if let Ok(id) = locked.first_set() {
            locked.clear(id);
            self.locked.set(locked);
            cs.deactivate();
            SystemCall(SysCallType::ProcessIdle(id));
        }
    }
}


pub struct Semaphore<'p> {
    locked: Option<&'p dyn Process>,
}

/* Le SysCalls non vanno... motivi sconosciuti. Indagare */
impl<'p> Semaphore<'p> {
    pub const fn new() -> Self {
        Self { locked: None }
    }

    pub fn wait(&mut self, _cs: &CriticalSection) {
        let cs = CriticalSection::activate();
        if self.locked.is_some() {
            return;
        }
        self.locked = SCHEDULER.get(&cs).running;
        SCHEDULER.get(&cs).running_stop();
        //SystemCall(SysCallType::ProcessStop(self.locked.unwrap().prio()));
    }

    pub fn release(&mut self, _cs: &CriticalSection) {
        let cs = CriticalSection::activate();
        if let Some(locked) = self.locked {
            let prio = locked.prio();
            self.locked = None;
            SCHEDULER.get(&cs).process_idle(prio);
            //SystemCall(SysCallType::ProcessIdle(prio));
        }
    }
}

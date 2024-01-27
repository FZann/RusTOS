use core::cell::{Cell, UnsafeCell};

use crate::kernel::processes::Process;
use crate::kernel::scheduler::{SCHEDULER, Scheduler};
use crate::kernel::BitVec;

use super::{SysCallType, SystemCall, CriticalSection};

pub struct Semaphore {
    locked: Cell<BitVec>,
}

impl Semaphore {
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

    /// Serve????
    pub(crate) fn lock_id(&self, id: usize) {
        let cs = CriticalSection::activate();
        let mut locked = self.locked.get();
        locked.set(id);
        self.locked.set(locked);
        cs.deactivate();
        SystemCall(SysCallType::ProcessStop(id));
    }

}

pub struct Mutex<'p, T> {
    locker: Option<&'p dyn Process>,
    resource: UnsafeCell<T>,
    sem: Semaphore,
}

impl<'p, T> Mutex<'p, T> {
    pub const fn new(value: T) -> Self {
        Self { 
            locker: None,
            resource: UnsafeCell::new(value),
            sem: Semaphore::new(),
        }
    }

    pub fn acquire(&mut self) -> Result<&mut T, ()> {
        let cs = CriticalSection::activate();
        if self.locker.is_some() {
            self.sem.wait();
        }

        self.locker = SCHEDULER.get(&cs).running;
        unsafe { Ok(&mut *self.resource.get()) }
    }

    pub fn release(&mut self, _cs: &CriticalSection) -> Result<(), ()> {
        let cs = CriticalSection::activate();
        if let Some(locked) = self.locker {
            if locked.prio() == SCHEDULER.get(&cs).running_id() {
                self.locker = None;
                self.sem.release();
                return Ok(());
            }
        }
        Err(())
    }
}

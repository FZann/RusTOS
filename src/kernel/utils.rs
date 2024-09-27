
use core::cell::UnsafeCell;

use crate::kernel::tasks::Process;
use crate::bitvec::BitVec;

use super::{SysCallType, SystemCall};

pub struct Semaphore {
    locked: BitVec,
}

impl Semaphore {
    pub const fn new() -> Self {
        Self {
            locked: BitVec::new(),
        }
    }

    pub fn wait(&mut self, task: &dyn Process) {
        let id = task.prio();
        self.locked.set(id);
        SystemCall(SysCallType::ProcessStop(id));
    }

    pub fn release(&mut self) {
        if let Ok(id) = self.locked.find_first_set() {
            self.locked.clear(id);
            SystemCall(SysCallType::ProcessIdle(id));
        }
    }

    /// Serve????
    pub(crate) fn lock_id(&mut self, id: usize) {
        self.locked.set(id);
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

    pub fn acquire(&mut self, task: &'p dyn Process) -> &mut T {
        if self.locker.is_some() {
            self.sem.wait(task);
        }

        self.locker = Some(task);
        unsafe { &mut *self.resource.get() }
    }

    pub fn release(&mut self, task: &dyn Process) {
        if let Some(locked) = self.locker {
            if locked.prio() == task.prio() {
                self.locker = None;
                self.sem.release();
            }
        }
    }
}


/// Coda. L'implementazione sul passaggio dei dati by-value (copia)
/// e non by-ref (puntatore/riferimento).
pub struct Queue<T: Sized + Copy, const SIZE: usize> {
    buf: [Option<T>; SIZE],
    head: usize,
    tail: usize,
    cnt: usize,
    push: Semaphore,
    pop: Semaphore,
}

impl<T, const SIZE: usize> Queue<T, SIZE>
where
    T: Sized + Copy,
{
    pub const fn new() -> Self {
        Self {
            buf: [None; SIZE],
            head: 0,
            tail: 0,
            cnt: 0,
            push: Semaphore::new(),
            pop: Semaphore::new(),
        }
    }

    pub fn push(&mut self, task: &dyn Process, object: T) {
        // Andiamo in attesa col semaforo, perché la coda è piena
        if self.cnt == SIZE {
            self.push.wait(task);
        }

        self.buf[self.head] = Some(object);
        self.head += 1;
        self.cnt += 1;
        if self.head >= SIZE {
            self.head = 0;
        }

        self.pop.release(); // Segnalazione per eventuali pop in attesa
    }

    pub fn pop(&mut self, task: &dyn Process, object: &mut T) {
        // Andiamo in attesa col semaforo, perché la coda è vuota
        if self.cnt == 0 {
            //self.pop.wait(task);
        }

        // Unwrap non panica sicuramente, abbiamo fatto il test prima!
        if let Some(obj) = self.buf[self.tail].take() {
            *object = obj;
            self.tail += 1;
            self.cnt -= 1;
            if self.tail >= SIZE {
                self.tail = 0;
            }
        }
        
        //self.push.release(); // Segnalazione per eventuali push in attesa
    }
}


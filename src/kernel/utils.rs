
use core::cell::{Cell, UnsafeCell};

use crate::kernel::tasks::Process;
use crate::bitvec::BitVec;

use super::{tasks::KERNEL, CritSect};

pub struct Semaphore {
    locked: Cell<BitVec>,
}

impl Semaphore {
    pub const fn new() -> Self {
        Self {
            locked: Cell::new(BitVec::new()),
        }
    }

    pub fn wait(&self, task: &dyn Process) {
        let cs = CritSect::activate();
        let mut locked = self.locked.get();
        let id = task.prio();
        locked.set(id);
        self.locked.set(locked);
        KERNEL.get(&cs).process_stop(id);
        cs.deactivate();
    }

    pub fn release(&self) {
        let cs = CritSect::activate();
        let mut locked = self.locked.get();

        if let Ok(id) = locked.find_first_set() {
            locked.clear(id);
            self.locked.set(locked);
            KERNEL.get(&cs).process_idle(id);
            cs.deactivate();
        }
    }

    /// Serve????
    pub(crate) fn lock_id(&self, id: usize) {
        let cs = CritSect::activate();
        let mut locked = self.locked.get();
        locked.set(id);
        self.locked.set(locked);
        KERNEL.get(&cs).process_stop(id);
        cs.deactivate();
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
pub struct Queue<T, const SIZE: usize> {
    sem: Semaphore,
    buf: [Option<T>; SIZE],
    head: usize,
    tail: usize,
}

impl<T, const SIZE: usize> Queue<T, SIZE>
where
    T: Sized + Copy,
{
    pub const fn new() -> Self {
        Self {
            sem: Semaphore::new(),
            buf: [None; SIZE],
            head: 0,
            tail: 0,
        }
    }

    pub fn push(&mut self, task: &dyn Process, object: T) {
        // Andiamo in attesa col semaforo, perché la coda è piena
        while self.buf[self.head].is_some() {
            self.sem.wait(task);
        }

        self.buf[self.head] = Some(object);
        self.head += 1;
        if self.head >= SIZE {
            self.head = 0;
        }

        self.sem.release(); // Segnalazione per eventuali pop in attesa
    }

    pub fn pop(&mut self, task: &dyn Process) -> T {
        // Andiamo in attesa col semaforo, perché la coda è vuota
        if self.buf[self.tail].is_none() {
            self.sem.wait(task);
        }

        // Unwrap non panica sicuramente, abbiamo fatto il test prima!
        let result = self.buf[self.tail].take().unwrap();
        self.tail += 1;
        if self.tail >= SIZE {
            self.tail = 0;
        }
        
        self.sem.release(); // Segnalazione per eventuali push in attesa

        result
    }
}


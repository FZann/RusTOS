
use core::cell::UnsafeCell;

use crate::kernel::{KERNEL, TCB};
use crate::bitvec::BitVec;


pub struct Semaphore {
    locked: BitVec,
}

impl Semaphore {
    pub const fn new() -> Self {
        Self {
            locked: BitVec::new(),
        }
    }

    pub fn wait(&mut self, task: &TCB) {
        let id = task.prio();
        self.locked.set(id);
        unsafe { KERNEL.get_unsafe().process_stop(id) };
    }

    pub fn release(&mut self) {
        if let Ok(id) = self.locked.find_first_set() {
            self.locked.clear(id);
            unsafe { KERNEL.get_unsafe().process_idle(id) };
        }
    }

    /// Serve????
    pub(crate) fn lock_id(&mut self, id: usize) {
        self.locked.set(id);
        unsafe { KERNEL.get_unsafe().process_stop(id) };
    }

}

pub struct Mutex<'p, T> {
    locker: Option<&'p TCB<'p>>,
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

    pub fn acquire(&mut self, task: &'p TCB) -> &mut T {
        if self.locker.is_some() {
            self.sem.wait(task);
        }

        self.locker = Some(task);
        unsafe { &mut *self.resource.get() }
    }

    pub fn release(&mut self, task: &TCB) {
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

    pub fn push(&mut self, task: &TCB, object: T) {
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

    pub fn pop(&mut self, task: &TCB, object: &mut T) {
        // Andiamo in attesa col semaforo, perché la coda è vuota
        if self.buf[self.tail].is_none() {
            self.sem.wait(task);
        }

        // Unwrap non panica sicuramente, abbiamo fatto il test prima!
        *object = self.buf[self.tail].take().unwrap();
        self.tail += 1;
        if self.tail >= SIZE {
            self.tail = 0;
        }
        
        self.sem.release(); // Segnalazione per eventuali push in attesa
    }
}


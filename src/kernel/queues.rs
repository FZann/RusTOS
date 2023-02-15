use crate::kernel::semaphores::Semaphore;


/// Coda. L'implementazione sul passaggio dei dati by-value (copia)
/// e non by-ref (puntatore/riferimento).
pub struct Queue<T, const SIZE: usize> {
    sem: Semaphore,
    buf: [Option<T>; SIZE],
    pop_id: usize,
    push_id: usize,
}

impl<T, const SIZE: usize> Queue<T, SIZE>
where
    T: Sized + Copy,
{
    const NONE: Option<T> = None;

    pub const fn allocate() -> Self {
        Self {
            sem: Semaphore::new(),
            buf: [Self::NONE; SIZE],
            pop_id: 0,
            push_id: 0,
        }
    }
    
    pub fn push(&mut self, object: T) {
        // Andiamo in attesa col semaforo, perché la coda è piena
        while self.buf[self.push_id].is_some() {
            self.sem.wait();
        }

        self.buf[self.push_id] = Some(object);
        self.sem.release();     // Segnalazione per eventuali pop in attesa
        self.push_id += 1;
        if self.push_id >= SIZE {
            self.push_id = 0;
        }
    }

    pub fn pop(&mut self) -> T {
        // Andiamo in attesa col semaforo, perché la coda è vuota
        while self.buf[self.pop_id].is_none() {
            self.sem.wait();
        }

        // Unwrap non panica sicuramente, abbiamo fatto il test prima!
        let result = self.buf[self.pop_id].take().unwrap();
        self.sem.release();     // Segnalazione per eventuali push in attesa
        self.pop_id += 1;
        if self.pop_id >= SIZE {
            self.pop_id = 0;
        }

        result
    }
    
}

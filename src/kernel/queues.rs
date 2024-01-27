use super::semaphores::Semaphore;


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

    pub fn push(&mut self, object: T) {
        // Andiamo in attesa col semaforo, perché la coda è piena
        while self.buf[self.head].is_some() {
            self.sem.wait();
        }

        self.buf[self.head] = Some(object);
        self.head += 1;
        if self.head >= SIZE {
            self.head = 0;
        }
        self.sem.release(); // Segnalazione per eventuali pop in attesa
    }

    pub fn pop(&mut self) -> T {
        // Andiamo in attesa col semaforo, perché la coda è vuota
        if self.buf[self.tail].is_none() {
            self.sem.wait();
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

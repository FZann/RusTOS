use crate::kernel::semaphores::Semaphore;

pub struct Queue<T, const SIZE: usize> {
    sem: Semaphore,
    buf: [Option<T>; SIZE],
    pop_index: usize,
    push_index: usize,
}

impl<T, const SIZE: usize> Queue<T, SIZE>
where
    T: Sized + Copy,
{
    const NONE: Option<T> = None;

    pub fn allocate() -> Self {
        Self {
            sem: Semaphore::new(),
            buf: [Self::NONE; SIZE],
            pop_index: 0,
            push_index: 0,
        }
    }

    pub fn push(&mut self, object: T) {
        // Andiamo in attesa col semaforo, perché la coda è piena
        while self.buf[self.push_index + 1].is_some() {
            self.sem.wait()
        }

        self.buf[self.push_index] = Some(object);
        self.sem.release();
        self.push_index += 1;
        if self.push_index >= SIZE {
            self.push_index = 0;
        }
    }

    pub fn pop(&mut self) -> T {
        while self.buf[self.pop_index].is_none() {
            self.sem.wait()
        }

        // Siamo sicuri che unwrap non panichi
        let result = self.buf[self.pop_index].take().unwrap();
        self.sem.release();
        self.pop_index += 1;
        if self.pop_index >= SIZE {
            self.pop_index = 0;
        }

        result
    }
}

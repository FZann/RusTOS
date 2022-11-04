use crate::kernel::semaphores::Semaphore;

pub struct Queue<T, const SIZE: usize> {
    sem: Semaphore,
    buf: [Option<T>; SIZE],
    pop_index: usize,
    push_index: usize,
}

impl<T: Sized, const SIZE: usize> Queue<T, SIZE> {
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
        
    }

    pub fn pop(&mut self) -> T {

    }
}

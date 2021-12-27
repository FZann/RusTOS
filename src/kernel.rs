use core::cell::Cell;

// Aliasing per poter usare la compilazione condizionale
pub use self::assembly::svc as SystemCall;

mod assembly;
mod vectors;

pub mod processes;
pub mod scheduler;

#[derive(PartialEq, PartialOrd)]
pub enum SysCallType {
    Nop,
    ProcessIdle,
    ProcessSleep(Ticks),
    ProcessStop,
    StartScheduler,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Ticks(Cell<usize>);

impl Ticks {
    pub const fn new(ticks: usize) -> Self {
        Ticks(Cell::new(ticks))
    }

    pub fn increment(&self) {
        self.0.set(self.0.get() + 1);
    }

    pub fn decrement(&self) {
        self.0.set(self.0.get() - 1);
    }

    pub fn set(&self, ticks: usize) {
        self.0.set(ticks);
    }

    pub fn value(&self) -> usize {
        self.0.get()
    }
}

impl core::ops::Add for Ticks {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.0.get() + rhs.0.get())
    }
}

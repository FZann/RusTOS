use core::cell::Cell;

// Aliasing per poter usare la compilazione condizionale
pub use self::armv7em_arch::sleep_cpu;
pub use self::armv7em_arch::svc as SystemCall;
pub use self::armv7em_arch::ExceptionFrame;

mod armv7em_arch;

pub mod processes;
pub mod scheduler;
pub mod semaphores;

pub type Ticks = usize;
pub type TaskHandle = fn() -> !;

#[derive(PartialEq, PartialOrd)]
pub enum SysCallType {
    Nop,
    ProcessIdle,
    ProcessSleep(Ticks),
    ProcessStop,
    StartScheduler,
}

#[inline(always)]
pub fn idle() {
    SystemCall(SysCallType::ProcessIdle);
}

#[inline(always)]
pub fn sleep(ticks: Ticks) {
    SystemCall(SysCallType::ProcessSleep(ticks));
}

#[inline(always)]
pub fn stop() {
    SystemCall(SysCallType::ProcessStop);
}

#[derive(Clone)]
pub struct BooleanVector {
    vec: Cell<usize>,
}

impl BooleanVector {
    pub const fn new() -> Self {
        BooleanVector { vec: Cell::new(0) }
    }

    pub fn read(&self, bit: u8) -> bool {
        self.vec.get() & (1 << bit) == (1 << bit)
    }

    pub fn set(&self, bit: u8) {
        let mut vec = self.vec.get();
        vec |= 1 << bit;
        self.vec.set(vec);
    }

    pub fn clear(&self, bit: u8) {
        let mut vec = self.vec.get();
        vec &= !(1 << bit);
        self.vec.set(vec);
    }

    pub fn value(&self) -> usize {
        self.vec.get()
    }
}

impl core::ops::BitOr for BooleanVector {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            vec: Cell::new(self.value() | rhs.value()),
        }
    }
}

pub mod processes;
pub mod queues;
pub mod scheduler;
pub mod semaphores;

mod armv7em_arch;

// Aliasing per poter usare la compilazione condizionale
pub use self::armv7em_arch::load_first_process;
pub use self::armv7em_arch::sleep_cpu;
pub use self::armv7em_arch::svc as SystemCall;
pub use self::armv7em_arch::ExceptionFrame;

pub type Ticks = usize;

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

#[derive(Clone, Copy)]
pub struct BitVec {
    vec: usize,
}

impl BitVec {
    pub const fn new() -> Self {
        Self { vec: 0 }
    }

    pub fn get(&self, bit: usize) -> bool {
        self.vec & (1 << bit) != 0
    }

    pub fn set(&mut self, bit: usize) {
        self.vec |= 1 << bit;
    }

    pub fn clear(&mut self, bit: usize) {
        self.vec &= !(1 << bit);
    }

    /// La funzione riporta un risultato 0-indexed, cioè ritorna 0
    /// se il primissimo bit è settato; in questo modo possiamo usare
    /// il valore per indirizzare gli array senza sottrazioni.
    pub fn first_set(&self) -> Result<usize, ()> {
        let zeros = self.vec.leading_zeros() as usize;
        if zeros == 32 {
            Err(())
        } else {
            Ok(31 - zeros)
        }
    }
}

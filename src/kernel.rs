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

type BitVec = usize;

trait BitVector {
    fn get(&self, bit: usize) -> bool;
    fn set(&mut self, bit: usize);
    fn clear(&mut self, bit: usize);
    fn first_set(&self) -> Result<usize, ()>;
}

impl BitVector for BitVec {
    fn get(&self, bit: usize) -> bool {
        self & (1 << bit) != 0
    }

    fn set(&mut self, bit: usize) {
        *self |= 1 << bit;
    }

    fn clear(&mut self, bit: usize) {
        *self &= !(1 << bit);
    }

    /// La funzione riporta un risultato 0-indexed, cioè ritorna 0
    /// se il primissimo bit è settato; in questo modo possiamo usare
    /// il valore per indirizzare gli array senza sottrazioni.
    fn first_set(&self) -> Result<usize, ()> {
        let zeros = self.leading_zeros() as usize;
        if zeros == 32 {
            Err(())
        } else {
            Ok(31 - zeros)
        }
    }

}
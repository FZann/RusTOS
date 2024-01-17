pub mod processes;
pub mod queues;
pub mod scheduler;
pub mod semaphores;
pub mod registers;

mod armv7em_arch;

use core::cell::UnsafeCell;

// Aliasing per poter usare la compilazione condizionale
pub(crate) use self::armv7em_arch::idle_task;
pub use self::armv7em_arch::load_first_process;
pub use self::armv7em_arch::svc as SystemCall;
pub use self::armv7em_arch::ExceptionFrame;
pub use self::armv7em_arch::request_context_switch;
use self::armv7em_arch::{interrupt_disable, interrupt_enable};

pub type Ticks = usize;


#[derive(Clone, Copy)]
pub enum SysCallType {
    Nop,
    ProcessIdle,
    ProcessSleep(Ticks),
    ProcessStop,
    StartScheduler,
}

#[repr(C)]
pub enum HardFaultError {
    FromProcess = 1,
    FromPrivileged = 2,
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

    /// Itera su tutti i bit settati del vettore, eseguendo la closure
    /// con l'indice del bit attivato.
    pub fn iter_on_set<F: Fn(usize)>(&self, f: F) {
        let mut vec = self.clone();
        while let Ok(id) = vec.first_set() {
            f(id);
            vec.clear(id);
        }
    }
}


/// Struttura zero-sized che garantisce al kernel l'accesso alle periferiche/globali
/// eliminando le data-races. 
/// L'implementazione disattiva gli interrupt. Per un sistema single-core è sufficiente, ma
/// non è adatta ai sistemi multicore.
pub struct CriticalSection;

impl CriticalSection {
    pub fn activate() -> CriticalSection {
        interrupt_disable();
        CriticalSection
    }
}

impl Drop for CriticalSection {
    fn drop(&mut self) {
        interrupt_enable();
    }
}

pub struct SyncCell<T> {
    obj: UnsafeCell<T>
}

unsafe impl<T> Sync for SyncCell<T> {}

impl<T> SyncCell<T> {
    pub const fn new(obj: T) -> Self {
        Self { 
            obj: UnsafeCell::new(obj),
        }
    }

    pub fn get_access<'cs>(&'cs self, _cs: &'cs CriticalSection) -> &'cs mut T {
        unsafe { &mut (*self.obj.get()) }
    }

    pub fn with(&self, mut f: impl FnMut(&mut T, &CriticalSection)) {
        let cs: CriticalSection = CriticalSection::activate();
        f(self.get_access(&cs), &cs);
        drop(cs);
    }
}


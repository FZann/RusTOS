pub mod processes;
pub mod queues;
pub mod scheduler;
pub mod semaphores;

mod armv7em_arch;

// Aliasing per poter usare la compilazione condizionale
pub use self::armv7em_arch::idle_task;
pub use self::armv7em_arch::load_first_process;
pub use self::armv7em_arch::svc as SystemCall;
pub use self::armv7em_arch::ExceptionFrame;
use cortex_m::interrupt::disable as interrupt_disable;
use cortex_m::interrupt::enable as interrupt_enable;

use core::cell::Cell;

pub type Ticks = usize;

#[derive(PartialEq, PartialOrd, Clone, Copy)]
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

/// Astrazione per rendere Sync-safe le shared globals.
/// In questo modo possiamo accedere a delle static, renderle mutabili
/// e accedere ai metodi mutabili.
/// E' Sync-safe siccome siamo su un sistema mono-core. Disabilitando gli
/// interrupt rende impossibile la modifica concorrenziale dei dati.
pub struct SyncShare<T> {
    obj: Cell<T>,
}

pub trait Syncable {}
unsafe impl<T> Sync for SyncShare<T> {}

impl<T: Syncable> SyncShare<T> {
    pub const fn new(obj: T) -> Self {
        Self {
            obj: Cell::new(obj)
        }
    }

    /// Critical Section. ciò che viene eseguito all'interno della CS
    /// non può essere interrotto da interrupt asincroni.
    pub fn cs(&self, mut f: impl FnMut(&mut T))  {
        interrupt_disable();
        unsafe { 
            f(&mut *self.obj.as_ptr());
            interrupt_enable();
        };
    }
}

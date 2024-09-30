mod armv7em_arch;

pub mod tasks;
pub mod utils;

use core::cell::UnsafeCell;

// Aliasing per poter usare la compilazione condizionale
pub(crate) use self::armv7em_arch::idle_task;
pub use self::armv7em_arch::SystemCall;
pub use self::armv7em_arch::ExceptionFrame;
pub(crate) use self::armv7em_arch::core_peripherals::CorePeripherals as CorePeripherals;
use self::armv7em_arch::{interrupt_disable, interrupt_enable};

pub type Ticks = usize;

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub enum SysCallType {
    Nop,
    StartScheduler,
    ContextSwith,
}


#[repr(C)]
#[derive(PartialEq, Eq)]
pub enum ExecContext {
    Privileged = 0,
    Process = 1,
    Error = 2,
}

impl From<usize> for ExecContext {
    fn from(value: usize) -> Self {
        match value {
            0 => ExecContext::Privileged,
            1 => ExecContext::Process,
            _ => ExecContext::Error,
        }
    }
}

impl ExecContext {
    pub fn is_privileged(&self) -> bool {
        *self == ExecContext::Privileged
    }

    pub fn is_process(&self) -> bool {
        *self == ExecContext::Process
    }
}

/// Astrazione per rendere Sync-safe le shared globals.
/// In questo modo possiamo accedere a delle static, renderle mutabili
/// e accedere ai metodi mutabili.
/// E' Sync-safe siccome siamo su un sistema mono-core. Disabilitando gli
/// interrupt rende impossibile la modifica concorrenziale dei dati.
pub struct CritCell<T: Sized>(UnsafeCell<T>);

impl<T: Sized> CritCell<T> {
    pub const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }

    pub fn with(&self, f: impl FnOnce(&mut T)) {
        interrupt_disable();
        unsafe { 
            f(&mut *self.0.get());
            interrupt_enable();
        };
    }

    pub fn get (&self, _cs: &CritSect) -> &mut T {
        unsafe { &mut (*self.0.get()) }
    }

    pub unsafe fn get_unsafe(&self) -> &mut T {
        &mut *self.0.get()
    }
}

unsafe impl<T: Sized> Sync for CritCell<T> {}

/// Token per l'abilitazione di una Critical Section
/// Creare una CritSect disabilita gli interrupts,
/// mentre il metodo Drop li riabilita
#[must_use]
pub struct CritSect;

impl CritSect {
    pub fn activate() -> Self {
        interrupt_disable();
        CritSect
    }
    
    fn deactivate(self) {
        drop(self);
    }
}

impl Drop for CritSect {
    fn drop(&mut self) {
        interrupt_enable();
    }
}
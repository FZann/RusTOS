pub mod tasks;

mod armv7em_arch;

// Aliasing per poter usare la compilazione condizionale
pub(crate) use self::armv7em_arch::idle_task;
pub use self::armv7em_arch::load_first_process;
pub use self::armv7em_arch::svc as SystemCall;
pub use self::armv7em_arch::ExceptionFrame;
use cortex_m::interrupt::disable as interrupt_disable;
use cortex_m::interrupt::enable as interrupt_enable;

pub type Ticks = usize;

#[derive(PartialEq, PartialOrd, Clone, Copy)]
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

/// Astrazione per rendere Sync-safe le shared globals.
/// In questo modo possiamo accedere a delle static, renderle mutabili
/// e accedere ai metodi mutabili.
/// E' Sync-safe siccome siamo su un sistema mono-core. Disabilitando gli
/// interrupt rende impossibile la modifica concorrenziale dei dati.
pub trait Syncable: Sync {
    fn cs(&self, f: impl FnOnce(&mut Self)) {
        interrupt_disable();
        unsafe { 
            //f(&mut *(self as *const Self as *mut Self));
            interrupt_enable();
        };
    }
}
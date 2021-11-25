#![no_std]
#![no_main]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(core_intrinsics)]

use core::panic::PanicInfo;

pub mod kernel;

#[panic_handler]
pub fn panic_handler(_: &PanicInfo) -> ! {
    loop {}
}

/// Stack frame hardware salvata dai Cortex-M
/// Permette di visualizzare i valori dei registri durante l'ultimo errore
#[repr(C)]
pub struct ExceptionFrame {
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    r12: u32,
    lr: u32,
    pc: u32,
    xpsr: u32,
}

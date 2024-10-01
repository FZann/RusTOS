#![no_std]
#![no_main]
#![feature(naked_functions)]
#![allow(dead_code)]

pub mod kernel;
pub mod peripherals;
pub mod bitvec;

use core::panic::PanicInfo;

use kernel::{tasks::{TCB, KERNEL}, ExceptionFrame, ExecContext};

#[panic_handler]
pub fn panic_handler(_: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
#[allow(non_snake_case)]
#[inline(always)]
fn OSFault(_frame: &ExceptionFrame, ctx: ExecContext, running: &mut TCB) {
    if ctx == ExecContext::Privileged {
        //panic!("From kernel!");
    };

    // Reimpostiamo il task che ha dato rogne
    running.setup();

    // Qui non possiamo essere interrotti: HardFault ha la massima priorità a livello HW
    // Quindi possiamo prendere la &mut del KERNEL senza CriticalSection
    unsafe { 
        KERNEL.get_unsafe().load_first_process();
    };
}

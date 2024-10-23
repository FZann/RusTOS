#![no_std]
#![no_main]
#![feature(naked_functions)]
#![allow(dead_code)]

pub mod kernel;
pub mod peripherals;
pub mod bitvec;

use core::panic::PanicInfo;

use kernel::{ExceptionFrame, ExecContext, KERNEL, TCB};

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

    running.stop();

    // Qui non possiamo essere interrotti: HardFault ha la massima priorità a livello HW
    // Quindi possiamo prendere la &mut del KERNEL senza CriticalSection
    unsafe { 
        KERNEL.get_unsafe().schedule_next();
    };
}

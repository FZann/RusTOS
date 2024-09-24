#![no_std]
#![no_main]
#![feature(naked_functions)]

pub mod kernel;
pub mod peripherals;
pub mod bitvec;

use core::panic::PanicInfo;

use kernel::{ExceptionFrame, HardFaultError};

#[panic_handler]
pub fn panic_handler(_: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn OSFault(_error: HardFaultError ,_frame: &ExceptionFrame) -> ! {
    loop {}
}

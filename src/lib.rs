#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(const_option)]
#![feature(const_nonnull_new)]

pub mod kernel;
pub mod peripherals;

use core::panic::PanicInfo;

use kernel::{ExceptionFrame, HardFaultError};


#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn OSFault(_error: HardFaultError ,_frame: &ExceptionFrame) -> ! {
    loop {}
}

#[panic_handler]
pub fn panic_handler(_: &PanicInfo) -> ! {
    loop {}
}

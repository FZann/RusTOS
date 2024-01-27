#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(const_option)]
#![feature(const_nonnull_new)]

pub mod kernel;
pub mod peripherals;

use core::panic::PanicInfo;

use kernel::{ExceptionFrame, HardFaultError};
use kernel::processes::Process;


#[no_mangle]
#[allow(non_snake_case)]
fn OSFault(_frame: &ExceptionFrame, error: HardFaultError, running: &dyn Process) -> ! {
    running.set_ticks(5);

    loop {}
}

#[panic_handler]
pub fn panic_handler(_: &PanicInfo) -> ! {
    loop {}
}

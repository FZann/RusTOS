#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(const_option)]
#![feature(const_nonnull_new)]

pub mod kernel;
pub mod peripherals;

use core::panic::PanicInfo;

#[panic_handler]
pub fn panic_handler(_: &PanicInfo) -> ! {
    loop {}
}

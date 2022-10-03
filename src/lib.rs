#![no_std]
#![no_main]
#![feature(naked_functions)]

use core::panic::PanicInfo;

pub mod kernel;

#[panic_handler]
pub fn panic_handler(_: &PanicInfo) -> ! {
    loop {}
}

#![no_std]
#![no_main]
#![feature(asm)]
#![feature(naked_functions)]

use core::panic::PanicInfo;

mod kernel;

#[panic_handler]
pub fn panic_handler(_: &PanicInfo) -> ! {
    loop {}
}

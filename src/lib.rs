#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(const_option)]
#![feature(const_nonnull_new)]

pub mod kernel;
pub mod peripherals;

use core::panic::PanicInfo;

use kernel::scheduler::{Scheduler, SCHEDULER};
use kernel::{ExceptionFrame, HardFaultError};
use kernel::processes::Process;

#[no_mangle]
#[allow(non_snake_case)]
#[inline(always)]
fn OSFault(_frame: &ExceptionFrame, error: HardFaultError, running: &mut dyn Process) {
    if error == HardFaultError::FromPrivileged {
        panic!("From kernel!");
    };

    // Reimpostiamo il task che ha dato rogne
    running.setup();
    unsafe { 
        SCHEDULER.unsafe_get().schedule_next();
    };
}

#[panic_handler]
pub fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}

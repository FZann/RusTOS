#![no_std]
#![no_main]

use RusTOS::kernel::processes::{PCB, Process, TaskHandle};
use RusTOS::{panic_handler, ExceptionFrame};
use RusTOS::kernel::scheduler::{SCHEDULER, Scheduler};



#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn OSEntry() -> ! {
    static mut STACK: [usize; 512] = [0usize; 512];

    unsafe {
        let pcb: PCB = Process::new(TaskHandle::new(ciao) , &mut STACK, 0);
        SCHEDULER.add_process(pcb);
        SCHEDULER.start();
    }
    loop {}
}

fn ciao() -> ! {
    loop {

    }
}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn OSFault(frame: &ExceptionFrame) -> ! {
    loop {}
}
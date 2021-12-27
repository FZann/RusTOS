#![no_std]
#![no_main]

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;
use RusTOS::kernel::processes::{Process, TaskHandle, PCB};
use RusTOS::kernel::scheduler::{Scheduler, SCHEDULER};
use RusTOS::kernel::{SysCallType, SystemCall, Ticks};
use RusTOS::{panic_handler, ExceptionFrame};

static mut STACK: [usize; 256] = [0usize; 256];
static mut STACK1: [usize; 256] = [0usize; 256];

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn OSEntry() -> ! {
    let mut sys_tick = cortex_m::peripheral::Peripherals::take().unwrap().SYST;
    sys_tick.set_clock_source(SystClkSource::Core);
    let reload = SYST::get_ticks_per_10ms() * 10;
    sys_tick.set_reload(reload);
    sys_tick.enable_interrupt();
    sys_tick.enable_counter();

    unsafe {
        let pcb: PCB = Process::new(TaskHandle::new(ciao), &mut STACK, 0);
        let pcb1: PCB = Process::new(TaskHandle::new(bello), &mut STACK1, 1);

        SCHEDULER.add_process(pcb);
        SCHEDULER.add_process(pcb1);
        SystemCall(SysCallType::StartScheduler);
        unreachable!();
    }
}

fn ciao() -> ! {
    let mut c = 0u32;
    loop {
        c += 1;
    }
}

fn bello() -> ! {
    unsafe {
        let rcc: *mut usize = 0x4002_1000 as *mut usize;
        let rcc = rcc.add(0x4C / 4);
        rcc.write(0x1);

        let gpioa: *mut usize = 0x4800_0000 as *mut usize;
        gpioa.write(0x400);
        let gpioa_out = gpioa.add(0x18 / 4) ;

        loop {
            gpioa_out.write(0x20);
            //PCB::sleep(Ticks::new(500));
            //gpioa_out.write(0x20_0000);
            //PCB::sleep(Ticks::new(500));
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn OSFault(frame: &ExceptionFrame) -> ! {
    loop {}
}

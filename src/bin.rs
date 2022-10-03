#![no_std]
#![no_main]

use RusTOS::kernel::semaphores::Semaphore;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;
use RusTOS::kernel::processes::{Process, PCB};
use RusTOS::kernel::scheduler::{Scheduler, SCHEDULER};
use RusTOS::kernel::{sleep_cpu, ExceptionFrame, SysCallType, SystemCall};

static mut IDLE_STACK: [usize; 256] = [0usize; 256];
static mut STACK: [usize; 256] = [0usize; 256];
static mut STACK1: [usize; 256] = [0usize; 256];

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn OSEntry() -> ! {
    let mut sys_tick = cortex_m::peripheral::Peripherals::take().unwrap().SYST;
    sys_tick.set_clock_source(SystClkSource::Core);
    let reload = SYST::get_ticks_per_10ms();
    sys_tick.set_reload(reload);
    sys_tick.enable_interrupt();
    sys_tick.enable_counter();

    unsafe {
        let idle: PCB = Process::new(idle_task, &mut IDLE_STACK, 0);
        let pcb: PCB = Process::new(ciao, &mut STACK, 1);
        let pcb1: PCB = Process::new(bello, &mut STACK1, 2);

        SCHEDULER.add_process(idle);
        SCHEDULER.add_process(pcb);
        SCHEDULER.add_process(pcb1);
        SystemCall(SysCallType::StartScheduler);
        unreachable!();
    }
}

fn idle_task() -> ! {
    loop {
        sleep_cpu();
    }
}

fn ciao() -> ! {
    let mut c = 0u32;
    loop {
        c += 1;
        PCB::sleep(50);
    }
}

fn bello() -> ! {
    let sem = Semaphore::new();
    unsafe {
        let mut rcc: *mut usize = 0x4002_1000 as *mut usize;
        rcc = rcc.add(5);
        let rccval = rcc.read();
        rcc.write(rccval | 1 << 17);

        let gpioa: *mut usize = 0x4800_0000 as *mut usize;

        gpioa.write(gpioa.read() | 1 << 10);
        let gpioa_out = gpioa.add(6);

        loop {
            gpioa_out.write(0x20);
            PCB::sleep(500);
            gpioa_out.write(0x20_0000);
            PCB::sleep(500);
            gpioa_out.write(0x20);
            PCB::sleep(500);
            sem.wait();
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn OSFault(_frame: &ExceptionFrame) -> ! {
    loop {}
}

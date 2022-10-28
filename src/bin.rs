#![no_std]
#![no_main]

use RusTOS::kernel::semaphores::Semaphore;
use RusTOS::kernel::processes::{Process, PCB};
use RusTOS::kernel::scheduler::{Scheduler, SCHEDULER};
use RusTOS::kernel::{sleep_cpu, ExceptionFrame, SysCallType, SystemCall, sleep};

static mut IDLE_STACK: [usize; 256] = [0; 256];
static mut STACK: [usize; 256] = [0; 256];
static mut STACK1: [usize; 256] = [0; 256];

static mut IDLE: Option<PCB> = None;
static mut CIAO: Option<PCB> = None;
static mut BELLO: Option<PCB> = None;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn OSEntry() -> ! {
    unsafe {
        IDLE = Some(PCB::new(idle_task, &mut IDLE_STACK, 0));
        CIAO = Some(PCB::new(ciao, &mut STACK, 1));
        BELLO = Some(PCB::new(bello, &mut STACK1, 2));


        SCHEDULER.add_process(IDLE.as_ref().unwrap());
        SCHEDULER.add_process(CIAO.as_ref().unwrap());
        SCHEDULER.add_process(BELLO.as_ref().unwrap());
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
        sleep(50);
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
            sleep(500);
            gpioa_out.write(0x20_0000);
            sleep(500);
            gpioa_out.write(0x20);
            sleep(500);
            //sem.wait();
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn OSFault(_frame: &ExceptionFrame) -> ! {
    loop {}
}

#![no_std]
#![no_main]

use RusTOS::kernel::processes::Task;
use RusTOS::kernel::scheduler::{Scheduler, SCHEDULER};
use RusTOS::kernel::semaphores::Semaphore;
use RusTOS::kernel::{sleep, sleep_cpu, ExceptionFrame, SysCallType, SystemCall};

static mut IDLE: Task<33> = Task::allocate();
static mut CIAO: Task<256> = Task::allocate();
static mut BELLO: Task<256> = Task::allocate();
static mut SEM: Semaphore = Semaphore::new();

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn OSEntry() -> ! {
    unsafe {
        IDLE.setup(idle_task, 0);
        CIAO.setup(ciao, 1);
        BELLO.setup(bello, 2);

        SCHEDULER.add_process(&IDLE);
        SCHEDULER.add_process(&CIAO);
        SCHEDULER.add_process(&BELLO);
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
        unsafe {
            SEM.release();
        }
        sleep(500);
    }
}

fn bello() -> ! {
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
            SEM.wait();
            //sleep(500);
            gpioa_out.write(0x20_0000);
            SEM.wait();
            //sleep(500);
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn OSFault(_frame: &ExceptionFrame) -> ! {
    loop {}
}

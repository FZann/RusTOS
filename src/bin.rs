#![no_std]
#![no_main]

use RusTOS::kernel::processes::Task;
use RusTOS::kernel::queues::Queue;
use RusTOS::kernel::scheduler::{Scheduler, SCHEDULER};
use RusTOS::kernel::semaphores::Semaphore;
use RusTOS::kernel::{sleep, sleep_cpu, ExceptionFrame, SysCallType, SystemCall};

static mut IDLE: Task<33> = Task::allocate(0);
static mut CIAO: Task<256> = Task::allocate(1);
static mut BELLO: Task<256> = Task::allocate(2);
static mut SEM: Semaphore = Semaphore::new();
static mut QUEUE: Queue<bool, 8> = Queue::allocate();

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn OSEntry() -> ! {
    unsafe {
        IDLE.setup(idle_task);
        CIAO.setup(ciao);
        BELLO.setup(bello);

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
            QUEUE.push(true);
            sleep(500);
            QUEUE.push(false);
            sleep(500);
        }
        
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
            let led_state = QUEUE.pop();
            match led_state {
                true => gpioa_out.write(0x20),
                false => gpioa_out.write(0x20_0000),
            }
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn OSFault(_frame: &ExceptionFrame) -> ! {
    loop {}
}


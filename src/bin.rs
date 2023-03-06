#![no_std]
#![no_main]

use RusTOS::kernel::processes::Task;
use RusTOS::kernel::queues::Queue;
use RusTOS::kernel::scheduler::{Scheduler, SCHEDULER};
use RusTOS::kernel::semaphores::{Semaphore, VecSemaphore};
use RusTOS::kernel::{sleep, ExceptionFrame, SysCallType, SystemCall};

static mut CIAO: Task<256> = Task::new(ciao, 0);
static mut BELLO: Task<256> = Task::new(bello, 1);
//static mut SEM: Semaphore = Semaphore::new();
static mut SEM: Semaphore = Semaphore::new();
//static mut QUEUE: Queue<u8, 8> = Queue::allocate();

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn OSEntry() -> ! {
    unsafe {
        SCHEDULER.add_process(&mut CIAO);
        SCHEDULER.add_process(&mut BELLO);
        SystemCall(SysCallType::StartScheduler);
        unreachable!();
    }
}

fn ciao() -> ! {
    let mut c = 0u32;
    loop {
        c += 1;
        unsafe {
            //QUEUE.push(1);
            sleep(500);
            //QUEUE.push(0);
            SEM.release();
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

        let mut led_state = false;
        loop {
            //let led_state = QUEUE.pop();
            led_state = !led_state;
            match led_state {
                true => gpioa_out.write(0x20),
                false => gpioa_out.write(0x20_0000),
            }
            SEM.wait();
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn OSFault(_frame: &ExceptionFrame) -> ! {
    loop {}
}

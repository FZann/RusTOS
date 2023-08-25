#![no_std]
#![no_main]

use RusTOS::kernel::processes::Task;
use RusTOS::kernel::queues::Queue;
use RusTOS::kernel::scheduler::{Scheduler, SCHEDULER};
use RusTOS::kernel::semaphores::Semaphore;
use RusTOS::kernel::{sleep, ExceptionFrame, SysCallType, SystemCall, HardFaultError, CriticalSection};

static mut CIAO: Task<256> = Task::new(ciao, 0);
static mut BELLO: Task<256> = Task::new(bello, 1);
static mut SEM: Semaphore = Semaphore::new();
//static QUEUE: SyncShare<Queue<u8, 8>> = Queue::new_syncable();


#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn OSEntry() -> ! {
    let s = SCHEDULER.get_access(&CriticalSection::activate());
    unsafe {
        let _ = s.add_process(&mut CIAO);
        let _ = s.add_process(&mut BELLO);
    }

    SystemCall(SysCallType::StartScheduler);
    unreachable!();
}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn OSFault(_error: HardFaultError ,_frame: &ExceptionFrame) -> ! {
    loop {}
}

fn ciao() -> ! {
    let mut c = 0u32;
    loop {
        unsafe {
            c += 1;
            //QUEUE.cs(|queue| queue.push(1));
            //sleep(500);
            //QUEUE.cs(|queue| queue.push(0));
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
        
        
        let mut led_state = false;
        loop {
            
            /* Quinto pin per il led (PIN 5) */
            //QUEUE.cs(|queue| led_state = queue.pop() == 1);
            led_state = !led_state;
            match led_state {
                _ => (),
            }
            SEM.wait();
        }
    }
}



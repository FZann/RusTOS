#![no_std]
#![no_main]

use RusTOS::kernel::processes::Task;
use RusTOS::kernel::queues::Queue;
use RusTOS::kernel::scheduler::{Scheduler, SCHEDULER};
use RusTOS::kernel::semaphores::Semaphore;
use RusTOS::kernel::{sleep, ExceptionFrame, SysCallType, SystemCall, HardFaultError, SyncCell};


static mut CIAO: Task<256> = Task::new(ciao, 0);
static mut BELLO: Task<256> = Task::new(bello, 1);
static SEM: SyncCell<Semaphore> = SyncCell::new(Semaphore::new());
//static QUEUE: SyncShare<Queue<u8, 8>> = Queue::new_syncable();


#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn OSEntry() -> ! {
    unsafe {
        let _ = SCHEDULER.add_process(&mut CIAO);
        let _ = SCHEDULER.add_process(&mut BELLO);
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
        c += 1;
        //QUEUE.cs(|queue| queue.push(1));
        //sleep(500);
        //QUEUE.cs(|queue| queue.push(0));
        SEM.with(|s, cs| s.release(cs));
        sleep(1000);
        //task.sleep(1000);
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
            
            /* Quinto pin per il led (PIN 5) */
            //QUEUE.cs(|queue| led_state = queue.pop() == 1);
            led_state = !led_state;
            match led_state {
                true => gpioa_out.write(0x20),
                false => gpioa_out.write(0x20_0000),
            }
            SEM.with(|s, cs| s.wait(cs));
        }
    }
}

#![no_std]
#![no_main]

use RusTOS::kernel::processes::{Task, Process};
use RusTOS::kernel::queues::Queue;
use RusTOS::kernel::registers::Peripheral;
use RusTOS::kernel::scheduler::Scheduler;
use RusTOS::kernel::semaphores::Semaphore;
use RusTOS::kernel::{CriticalSection, SysCallType, SystemCall};
use RusTOS::kernel::SyncCell;

use RusTOS::peripherals::gpio::GPIOA;

static CIAO: SyncCell<Task<256>> = SyncCell::new(Task::new(ciao, 0));
static BELLO: SyncCell<Task<256>> = SyncCell::new(Task::new(bello, 1));
static SEM: SyncCell<Semaphore> = SyncCell::new(Semaphore::new());
static QUEUE: SyncCell<Queue<u8, 8>> = SyncCell::new(Queue::new());

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn OSEntry() -> ! {
    let cs = CriticalSection::activate();
    let s  = RusTOS::kernel::scheduler::access_scheduler(&cs);
    let _ = s.add_process(CIAO.get(&cs));
    let _ = s.add_process(BELLO.get(&cs));
    cs.deactivate();
    
    SystemCall(SysCallType::StartScheduler);
    unreachable!();
}

fn ciao(task: &mut dyn Process) -> ! {
    let mut c = 0usize;
    loop {
        c += 1;
        let led = c % 2;
        //QUEUE.with(|queue, cs| queue.push(led as u8, cs));
        SEM.with(|s, cs| s.release(cs) );
        task.sleep(200);
        
    }
}

fn bello(task: &mut dyn Process) -> ! {
    unsafe {
        let mut rcc: *mut usize = 0x4002_1000 as *mut usize;
        rcc = rcc.add(5);
        let rccval = rcc.read();
        rcc.write(rccval | 1 << 17);        
    };
    
        GPIOA::with(|gpioa| gpioa.set_output(5));
        let mut led_state = false;
        loop {
            /* Quinto pin per il led (PIN 5) */
            //QUEUE.with(|queue, cs| led_state = queue.pop(cs) == 1);
            led_state = !led_state;
            match led_state {
                true => GPIOA::with(|gpioa| gpioa.set_high(5)),
                false => GPIOA::with(|gpioa| gpioa.set_low(5)),
            }
            
            //task.sleep(200);
            SEM.with(|s, cs | s.wait(cs));
    }
}

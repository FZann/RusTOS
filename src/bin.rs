#![no_std]
#![no_main]

use RusTOS::kernel::{CritCell, CritSect};
use RusTOS::kernel::tasks::{Process, Task, KERNEL};
use RusTOS::peripherals::gpio::GPIOA;
use RusTOS::peripherals::Peripheral;

static CIAO: CritCell<Task<128>> = CritCell::new(Task::new(ciao, 0));
static BELLO: CritCell<Task<128>> = CritCell::new(Task::new(bello, 1));

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn OSEntry() -> ! {
    let cs = CritSect::activate();
    let _ = KERNEL.get(&cs).add_process(CIAO.get(&cs));
    let _ = KERNEL.get(&cs).add_process(BELLO.get(&cs));
    KERNEL.get(&cs).init(cs);
}

fn ciao(task: &mut dyn Process) -> ! {
    let mut c = 0u32;
    loop {
        c += 1;
        //QUEUE.cs(|queue| queue.push(1));
        //sleep(500);
        //QUEUE.cs(|queue| queue.push(0));
        task.sleep(500);
    }
}

fn bello(task: &mut dyn Process) -> ! {
    unsafe {
        let mut rcc: *mut usize = 0x4002_1000 as *mut usize;
        rcc = rcc.add(5);
        let rccval = rcc.read();
        rcc.write(rccval | 1 << 17);
        
        GPIOA::regs().set_output(5);
        GPIOA::regs().set_low(5);
        
        let mut led_state = false;
        loop {
            
            /* Quinto pin per il led (PIN 5) */
            task.sleep(250);
            led_state = !led_state;
            match led_state {
                true => GPIOA::regs().set_high(5),
                false => GPIOA::regs().set_low(5),
            }
        }
    }
}



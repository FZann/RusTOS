#![no_std]
#![no_main]

use RusTOS::kernel::{CritCell, CritSect};
use RusTOS::kernel::tasks::{Process, Task, KERNEL};

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
        
        //PA5.set_dir(1);
        
        let mut led_state = false;
        loop {
            
            /* Quinto pin per il led (PIN 5) */
            //QUEUE.cs(|queue| led_state = queue.pop() == 1);
            led_state = !led_state;
            match led_state {
                true => (), //PA5.set_high(),
                false => (), //PA5.set_low(),
            }
        }
    }
}



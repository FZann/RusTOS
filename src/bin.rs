#![no_std]
#![no_main]

use RusTOS::kernel::{sleep, tasks::{Task, KERNEL}, Syncable, SysCallType, SystemCall};

static mut CIAO: Task<256> = Task::new(ciao, 0);
static mut BELLO: Task<256> = Task::new(bello, 1);


#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn OSEntry() -> ! {
    unsafe {
        KERNEL.add_process(&mut CIAO);
        KERNEL.add_process(&mut BELLO);
    };

    SystemCall(SysCallType::StartScheduler);
    unreachable!();
}

fn ciao() -> ! {
    let mut c = 0u32;
    loop {
        unsafe {
            c += 1;
            //QUEUE.cs(|queue| queue.push(1));
            //sleep(500);
            //QUEUE.cs(|queue| queue.push(0));
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



#![no_std]
#![no_main]

use RusTOS::kernel::utils::{Queue, Semaphore};
use RusTOS::kernel::{CritCell, CritSect};
use RusTOS::kernel::tasks::{Process, Task, KERNEL};
use RusTOS::peripherals::gpio::GPIOA;
use RusTOS::peripherals::Peripheral;

static CIAO: CritCell<Task<128>> = CritCell::new(Task::new(ciao, 0));
static BELLO: CritCell<Task<128>> = CritCell::new(Task::new(bello, 1));
static SEM: CritCell<Semaphore> = CritCell::new(Semaphore::new());
static QUEUE: CritCell<Queue<u32, 1>> = CritCell::new(Queue::new());

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

        
        //QUEUE.with(|queue| queue.push(task, c));
        SEM.with(|s| s.release());
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
        
        core::arch::asm!(
            "ldr    r0, =ld_data",
            "str    r3, [r0]",
        );


        loop {
            let q = QUEUE.get_unsafe();
            //let mut x = 0u32;
            //q.pop(task, &mut x);
            //led_state = x & 1 == 1;

            SEM.with(|s| s.wait(task));
            led_state = !led_state;
            match led_state {
                true => GPIOA::regs().set_high(5),
                false => GPIOA::regs().set_low(5),
            }
        }
    }
}



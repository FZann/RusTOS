//! RusTOS - Rust Real Time Operating System 
//! Copyright (C) 2025 - Fabio Zanin - fabio.zanin93@outlook.com
//! 
//! This program is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License.
//! 
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//! GNU General Public License for more details.
//! 
//! You should have received a copy of the GNU General Public License
//! along with this program.  If not, see <https://www.gnu.org/licenses/>.

#![no_std]
#![no_main]

use RusTOS::drivers::serial::SerialPort;
use RusTOS::drivers::serial::SerialStream;
use RusTOS::hal::gpio::*;
use RusTOS::hal::uart::*;
use RusTOS::hal::tim::*;
use RusTOS::kernel::*;

static CIAO_STACK: Stack::<256> = Stack::new();
static BELLO_STACK: Stack::<256> = Stack::new();
static UART_STACK: Stack::<256> = Stack::new();

static CIAO_TASK: Task = Task::new(ciao, 0, &CIAO_STACK);
static BELLO_TASK: Task = Task::new(bello, 1, &BELLO_STACK);
static UART_TASK: Task = Task::new(uart, 2, &UART_STACK);


static QUEUE: Queue<PinState, 8> = Queue::new();
static SB: StreamBuffer::<u8, 8, 2> = StreamBuffer::new();

static LED: CriticalCell<PA5<Output<PushPull>>> = CriticalCell::new(PA5::allocate());
//static LED: CriticalCell<PA5<Alternate<AF1>>> = CriticalCell::new(PA5::allocate());

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn OSEntry() -> ! {
    let cs = CritSect::activate();
    let k = KERNEL.access(&cs);
    let _ = k.add_task(&CIAO_TASK);
    let _ = k.add_task(&BELLO_TASK);
    
    #[cfg(feature = "timers")]
    let _ = k.new_timer(
        Duration::new(350),
        blink_led,
        true,
        TimerMode::CountedBurst(5, 60, 4),
    );
    
    let _ = LED.access(&cs).init().set_low();

    k.init(cs);
}

fn blink_led() {
    LED.with(|_, led| { let _ = led.toggle(); } );
}

fn ciao(task: &mut Task) -> ! {
    let mut c = 0u32;
    
    loop {
        if c < 3000 {
            c += 5;
        }

        QUEUE.push(task, PinState::High);

        task.sleep(c);
    }
}

fn bello(task: &mut Task) -> ! {
    let timeout = ms::new(100);
    let mut led_state;
    
    task.sleep(5000);

    let mut data = [0u8; 10usize];
    SB.read_available(&mut data);

    loop {
        if let Ok(x) = QUEUE.pop_timeout(task, timeout) {
            led_state = x;
        } else {
            led_state = PinState::Low;
        }

        LED.with(|_, led| { let _ = led.set_state(led_state); } );
    }
}

fn uart(task: &mut Task) -> ! {
    let mut serial = SerialPort::<UART1>::new();
    serial.setup(38400, SerialMode::Mode8N1, SerialProto::RS232);
    UART1::TX3.init();
    UART1::RX3.init();
    
    let mut data = [0u8; 10usize];

    for (id, byte) in data.iter_mut().enumerate() {
        *byte = id as u8 + '0' as u8;
    }

    SB.write(task, &data);

    loop {
        task.sleep(1000);
        serial.send(&data);
        serial.send(&['\n' as u8, '\r' as u8]);
    }
}
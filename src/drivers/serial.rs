use core::mem::MaybeUninit;
use core::ptr::NonNull;

use crate::hal::dma::DmaStream;
use crate::kernel::StreamBuffer;
use crate::hal::uart::*; 


pub trait SerialStream {
    fn setup(&mut self, baud: usize, mode: SerialMode, proto: SerialProto);
    fn send(&mut self, tx: &[u8]);
    fn read(&mut self, rx: &mut [u8]) -> usize;
}

/// Driver to use correctly a SerialPort
/// 
/// This class must be used wrapped in a Mutex, as it have mutable APIs that are not Sync-safe.
pub struct SerialPort<Port: Uart> {
    uart: MaybeUninit<Port>,
}

impl<Port: Uart> SerialPort<Port> {
    pub const fn new() -> Self {
        Self {
            uart: MaybeUninit::zeroed(),
        }
    }
}

impl<Port: Uart> SerialStream for SerialPort<Port> {
    fn setup(&mut self, baud: usize, mode: SerialMode, proto: SerialProto) {
        self.uart = MaybeUninit::new(Port::init(baud, mode, proto));
    }

    fn send(&mut self, tx: &[u8]) {
        for byte in tx {
            unsafe { self.uart.assume_init_mut() }.tx(*byte);
        }
    }

    fn read(&mut self, rx: &mut [u8]) -> usize {
        for byte in &mut *rx {
            *byte = unsafe { self.uart.assume_init_mut() }.rx();
        }

        rx.len()
    }
}


pub struct SerialPortISR<Port: Uart, const SIZE: usize, const TX_TRG: usize, const RX_TRG: usize> {
    uart: MaybeUninit<Port>,
    tx: StreamBuffer<u8, SIZE, TX_TRG>,
    rx: StreamBuffer<u8, SIZE, RX_TRG>,
}

impl<Port: Uart, const SIZE: usize, const TX_TRG: usize, const RX_TRG: usize> SerialPortISR<Port, SIZE, TX_TRG, RX_TRG> {
    pub const fn new<>() -> Self {
        Self {
            uart: MaybeUninit::zeroed(),
            tx: StreamBuffer::<u8, SIZE, TX_TRG>::new(),
            rx: StreamBuffer::<u8, SIZE, RX_TRG>::new(),
        }
    }
}

impl<Port: Uart, const SIZE: usize, const TX_TRG: usize, const RX_TRG: usize> SerialStream for SerialPortISR<Port, SIZE, TX_TRG, RX_TRG>  {
    fn setup(&mut self, baud: usize, mode: SerialMode, proto: SerialProto) {
        self.uart = MaybeUninit::new(Port::init(baud, mode, proto));
    }

    fn send(&mut self, tx: &[u8]) {
        todo!()
    }

    fn read(&mut self, rx: &mut [u8]) -> usize {
        todo!()
    }
}


pub struct SerialPortDMA<Port: Uart, const SIZE: usize> {
    uart: MaybeUninit<Port>,
    tx: NonNull<dyn DmaStream>,
    rx: NonNull<dyn DmaStream>,
    buff: [u8; SIZE],
    overflw: bool,
}

impl<Port: Uart, const SIZE: usize> SerialPortDMA<Port, SIZE> {
    pub const fn new(tx: &'static dyn DmaStream, rx: &'static dyn DmaStream) -> Self {
        Self {
            uart: MaybeUninit::zeroed(),
            tx: unsafe { NonNull::new_unchecked(tx as *const dyn DmaStream as *mut dyn DmaStream) },
            rx: unsafe { NonNull::new_unchecked(rx as *const dyn DmaStream as *mut dyn DmaStream) },
            buff: [0; SIZE],
            overflw: false,

        }
    }
}

impl<Port: Uart, const SIZE: usize> SerialStream for SerialPortDMA<Port, SIZE> {
    fn setup(&mut self, baud: usize, mode: SerialMode, proto: SerialProto) {
        self.uart = MaybeUninit::new(Port::init(baud, mode, proto));
    }

    fn send(&mut self, tx: &[u8]) {
        todo!()
    }

    fn read(&mut self, rx: &mut [u8]) -> usize {
        todo!()
    }
}

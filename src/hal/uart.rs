use crate::kernel::time::us;
use crate::hal::gpio::*;

pub use crate::hw::uart::*;

use super::dma::{DmaSink, DmaSource};

pub enum SerialMode {
    Mode7N1,
    Mode8N1,
    Mode9N1,
}

pub enum SerialProto {
    RS232,
    RS485,
    LIN
}

pub enum TRxType {
    LsbFirst,
    MsbFirst,
}

pub trait Uart: DmaSink + DmaSource {
    fn tx(&self, tx: u8);
    fn rx(&self) -> u8;

    fn rx_fifo_not_empty(&self) -> bool;
    fn tx_fifo_not_full(&self) -> bool;
    
    fn init(baud: usize, mode: SerialMode, proto: SerialProto) -> Self;
    fn activate(&mut self) -> &mut Self;
    fn deactivate(&mut self) -> &mut Self;
    fn set_baud(&mut self, baud: usize) -> &mut Self;
    fn set_mode(&mut self, mode: SerialMode) -> &mut Self;
    fn set_proto(&mut self, proto: SerialProto) -> &mut Self;
    fn set_msb(&mut self, typ: TRxType) -> &mut Self;

    fn set_tx_pin(pin: &mut impl PinSetup);
    fn set_rx_pin(pin: &mut impl PinSetup);
}


pub trait SerialTimeout {
    fn set_timeout(&mut self, timeout: us);
    fn remove_timeout(&mut self);
}

use core::marker::PhantomData;

use crate::kernel::registers::{RW, RO, WO};
use crate::make_peripheral;

pub trait OutputType {}

pub struct Input;
pub struct Output<TYPE: OutputType> { _type: PhantomData<TYPE>, }
pub struct PushPull;
pub struct OpenDrain;
pub struct NoPull;
pub struct PullDown;
pub struct PullUp;

impl OutputType for PushPull {}
impl OutputType for OpenDrain {}


#[repr(C)]
pub struct GpioReg {
    mode: RW<u32>,
    otype: RW<u32>,
    ospeed: RW<u32>,
    pupd: RW<u32>,
    id: RO<u32>,
    od: RW<u32>,
    bsr: WO<u32>,
    lck: RW<u32>,
    afl: RW<u32>,
    afh: RW<u32>,
    br: WO<u32>,
}

impl GpioReg {
    pub fn set_high(&mut self, n: usize) {
        unsafe { self.bsr.write(1 << n); }
    }

    pub fn set_low(&mut self, n: usize) {
        unsafe { self.bsr.write(1 << (n + 16)); }
    }

    pub fn set_input(&mut self, n: usize) {
        unsafe { self.mode.modify(|reg| reg & !(1 << (n + n))); }
    }

    pub fn set_output(&mut self, n: usize) {
        unsafe { self.mode.modify(|reg| reg | 1 << (n + n)); }
    }
}

make_peripheral!(GPIOA: 0x4800_0000, GpioReg);
make_peripheral!(GPIOB: 0x4800_0400, GpioReg);
make_peripheral!(GPIOC: 0x4800_0800, GpioReg);
make_peripheral!(GPIOD: 0x4800_0C00, GpioReg);


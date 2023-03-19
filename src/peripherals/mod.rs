use core::{marker::PhantomData, mem::transmute};

trait RegAddress {
    type Register;
    const ADDRESS: *mut Self::Register;
    fn as_mut_ref() -> &'static mut Self::Register {
        unsafe{ transmute(Self::ADDRESS)}
    }
}

#[repr(C)]
struct GpioReg {
    mode: usize,
    otype: usize,
    ospeed: usize,
    pupd: usize,
    id: usize,
    od: usize,
    bsr: usize,
    lck: usize,
    afl: usize,
    afh: usize,
    br: usize,
}

impl GPIO for GpioReg {
    fn set_high(&mut self) {
        self.bsr = 0x20;
    }

    fn set_low(&mut self) {
        self.bsr = 0x20_0000;
    }
}

pub struct GPIOA {
    _data: PhantomData<*const ()>
}

impl GPIOA {
    pub const fn new() -> Self {
        Self {
            _data: PhantomData,
        }
    }
}

impl RegAddress for GPIOA {
    type Register = GpioReg;
    const ADDRESS: *mut Self::Register = 0x4800_0000 as *mut _;
}

pub struct GPIOB {
    _data: PhantomData<*const ()>
}

impl GPIOB {
    pub const fn new() -> Self {
        Self {
            _data: PhantomData,
        }
    }
}

impl RegAddress for GPIOB {
    type Register = GpioReg;
    const ADDRESS: *mut Self::Register = 0x4800_0000 as *mut _;
}

pub trait GPIO {
    fn set_high(&mut self);
    fn set_low(&mut self);
}

impl<T> GPIO for T 
where 
    T: RegAddress + 'static,
    T::Register: GPIO,
    {
    fn set_high(&mut self) {
        let gpio = Self::as_mut_ref();
        gpio.set_high();
    }

    fn set_low(&mut self) {
        let gpio = Self::as_mut_ref();
        gpio.set_low();
    }
}
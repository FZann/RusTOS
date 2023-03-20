use core::{marker::PhantomData, mem::transmute};

trait MemMappedRegister {
    type Register;
    const ADDRESS: *mut Self::Register;
    fn as_mut_ref() -> &'static mut Self::Register {
        unsafe { transmute(Self::ADDRESS) }
    }
}

pub struct Input;
pub struct Output<TYPE> { _type: PhantomData<TYPE>, }
pub struct PushPull;
pub struct OpenDrain;
pub struct NoPull;
pub struct PullDown;
pub struct PullUp;

pub trait GpioPort {
    fn set_high(&mut self, n: usize);
    fn set_low(&mut self, n: usize);
    fn set_dir(&mut self, n: usize, dir: usize);
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

impl GpioPort for GpioReg {
    fn set_high(&mut self, n: usize) {
        self.bsr = 1 << n;
    }

    fn set_low(&mut self, n: usize) {
        self.bsr = 1 << (n + 16);
    }

    fn set_dir(&mut self, n: usize, dir: usize) {
        self.mode = dir << (n + n);
    }
}


pub trait GpioPin {
    fn set_high(&mut self);
    fn set_low(&mut self);
    fn set_dir(&mut self, dir: usize);
}

pub trait GpioNum {
    fn num(&self) -> usize;
}

impl<T> GpioPin for T
where
    T: GpioNum,
    T: MemMappedRegister + 'static,
    T::Register: GpioPort,
{
    fn set_high(&mut self) {
        Self::as_mut_ref().set_high(self.num());
    }

    fn set_low(&mut self) {
        Self::as_mut_ref().set_low(self.num());
    }

    fn set_dir(&mut self, dir: usize) {
        Self::as_mut_ref().set_dir(self.num(), 1);
    }
}

pub struct Pin<const N: usize, MODE, PULL> {
    _mode: PhantomData<MODE>,
    _pull: PhantomData<PULL>,
}

impl<const N: usize, MODE, PULL> Pin<N, MODE, PULL> {
    pub(crate) const fn new() -> Pin<N, Input, PullUp> {
        Pin::<N, Input, PullUp> {
            _mode: PhantomData,
            _pull: PhantomData,
        }
    }
}

impl<const N: usize, MODE, PULL> GpioNum for Pin<N, MODE, PULL> {
    fn num(&self) -> usize {
        N
    }
}

macro_rules! make_gpio {
    ($gpio: ident: $addr:expr, $([$pin: ident, $n: expr]),+) => {
        
        #[allow(non_snake_case)]
        pub mod $gpio {
            use super::{Pin, Input, PullUp};
            use super::{MemMappedRegister, GpioReg};

            $(pub static mut $pin: Pin<$n, Input, PullUp> = Pin::<$n, Input, PullUp>::new();


            impl<MODE, PULL> MemMappedRegister for Pin<$n, MODE, PULL> {
                type Register = GpioReg;
                const ADDRESS: *mut Self::Register = $addr as *mut _;
            })+
        }





    };
}


make_gpio!(GPIOA: 0x4800_0000, [PA5, 5], [PA1, 1]);
//make_gpio!(GPIOB: 0x4800_0400, [PB5, 5], [PB1, 1]);
//make_gpio!(GPIOB: 0x4800_0400, [PB5, 5], [PB1, 1]);
//make_gpio!(GPIOC: 0x4800_0800, [PB5, 5], [PB1, 1]);
//make_gpio!(GPIOD: 0x4800_0C00, [PB5, 5], [PB1, 1]);


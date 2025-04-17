pub use crate::kernel::time::Hz;
pub use crate::hw::tim::*;

pub enum TimMode {
    OneShot,
    Loop,
}

pub enum CountDir {
    Up,
    Down,
    UpDown,
}

pub trait TimeBase {
    type BITS;
    const DIR: CountDir;

    fn init_clock(&self);
    fn start(&mut self);
    fn stop(&mut self);
    fn set_mode(&mut self, mode: TimMode);
    fn set_frequency(&mut self, freq: Hz);
    fn activate_interrupt(&mut self);
    fn deactivate_interrupt(&mut self);
}

pub trait OutputCompare<const CH: usize>: TimeBase {
    const CH: usize = CH;

    fn set_compare_value(&mut self, comp: Self::BITS);
    fn set_compare_mode(&mut self, mode: CompareMode);
}

pub trait CompareComplementary<const CH: usize>: OutputCompare<CH> {
    fn set_complementary(&mut self, mode: ComplementaryMode);
}

pub trait PWM<const CH: usize>: OutputCompare<CH> {

}

pub trait InputCapture<const CH: usize>: TimeBase {

}

pub trait QuadratureEncoder: TimeBase {

}



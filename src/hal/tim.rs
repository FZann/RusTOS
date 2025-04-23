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



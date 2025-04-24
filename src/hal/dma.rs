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

use crate::kernel::registers::Peripheral;
pub use crate::hw::dma::*;


#[derive(Debug, Clone, Copy)]
pub struct DmaAddress(usize);

impl DmaAddress {
    pub const fn new(adr: usize) -> Self {
        Self(adr)
    }
}

impl From<usize> for DmaAddress {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<DmaAddress> for usize {
    fn from(value: DmaAddress) -> Self {
        value.0
    }
}

impl<T: Sized> From<&T> for DmaAddress {
    fn from(value: &T) -> Self {
        Self(value as *const T as usize)
    }
}

impl<T: Sized> From<&mut T> for DmaAddress {
    fn from(value: &mut T) -> Self {
        Self(value as *const T as usize)
    }
}

impl<T: Sized> From<*const T> for DmaAddress {
    fn from(value: *const T) -> Self {
        Self(value as usize)
    }
}

impl<T: Sized> From<*mut T> for DmaAddress {
    fn from(value: *mut T) -> Self {
        Self(value as usize)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DmaTrigger {
    SW,
    HW(usize),
}

impl DmaTrigger {
    pub const fn new(trg: usize) -> Self {
        Self::HW(trg)
    }
}

impl From<usize> for DmaTrigger {
    fn from(value: usize) -> Self {
        Self::HW(value)
    }
}


#[derive(Debug, Clone, Copy)]
pub enum DmaInterrupts {
    TRxComplete,
    HalfTRx,
    TRxError,
}

#[derive(Debug, Clone, Copy)]
pub enum DmaMode {
    SingleWord,
    SingleBlock,
    RepeatedWord,
    RepeatedBlock,
}



#[derive(Debug, Clone, Copy)]
pub enum DmaType {
    Peripheral,
    Memory,
}

pub trait DmaSink {
    fn get_addr(&self) -> DmaAddress;
    fn get_word_size(&self) -> DmaWordSize;
    fn get_size(&self) -> usize;
    fn get_target_type(&self) -> DmaType;
}

pub trait DmaSource {
    fn get_addr(&self) -> DmaAddress;
    fn get_word_size(&self) -> DmaWordSize;
    fn get_size(&self) -> usize;
    fn get_target_type(&self) -> DmaType;
}

pub(crate) trait DmaPeripheralSource: Peripheral  {
    const SRC: DmaAddress;
    const MODE: DmaMode;
    const TRG: DmaTrigger;
    const WORD: DmaWordSize;

    fn get_mode(&self) -> DmaMode {
        Self::MODE
    }

    fn get_trigger(&self) -> DmaTrigger {
        Self::TRG
    }

    fn get_word_size(&self) -> DmaWordSize {
        Self::WORD
    }
}

impl<T: DmaPeripheralSource> DmaSource for T {
    fn get_addr(&self) -> DmaAddress {
        Self::SRC
    }

    fn get_word_size(&self) -> DmaWordSize {
        self.get_word_size()
    }

    fn get_size(&self) -> usize {
        0
    }

    fn get_target_type(&self) -> DmaType {
        DmaType::Peripheral
    }
}

pub(crate) trait DmaPeripheralSink: Peripheral {
    const DST: DmaAddress;
    const MODE: DmaMode;
    const TRG: DmaTrigger;
    const WORD: DmaWordSize;

    fn get_mode(&self) -> DmaMode {
        Self::MODE
    }

    fn get_trigger(&self) -> DmaTrigger {
        Self::TRG
    }

    fn get_word_size(&self) -> DmaWordSize {
        Self::WORD
    }
}

impl<T: DmaPeripheralSink> DmaSink for T {
    fn get_addr(&self) -> DmaAddress {
        Self::DST
    }

    fn get_word_size(&self) -> DmaWordSize {
        self.get_word_size()
    }

    fn get_size(&self) -> usize {
        0
    }

    fn get_target_type(&self) -> DmaType {
        DmaType::Peripheral
    }
}

pub trait DmaStream {
    fn ch_id(&self) -> u8;
    fn enable_stream(&mut self);
    fn disable_stream(&mut self);
    fn set_transfer(&mut self, src: &dyn DmaSource, dst: &dyn DmaSink, mode: DmaMode, size: usize);
    fn enable_interrupt(&mut self, irq: DmaInterrupts);
    fn disable_interrupt(&mut self, irq: DmaInterrupts);
}


//*********************************************************************************************************************
// BUFFERS IMPLEMENTATION
//*********************************************************************************************************************
#[macro_export]
macro_rules! dma_buffer {
    ($($target:ty),+ => $size:expr) => {
        $(impl DmaSource for & $target {
            fn get_addr(&self) -> DmaAddress {
                self.as_ptr().into()
            }
        
            fn get_word_size(&self) -> DmaWordSize {
                $size
            }

            fn get_size(&self) -> usize {
                core::mem::size_of_val(self)
            }

            fn get_target_type(&self) -> DmaType {
                DmaType::Memory
            }
        }

        impl DmaSource for &mut $target {
            fn get_addr(&self) -> DmaAddress {
                self.as_ptr().into()
            }
        
            fn get_word_size(&self) -> DmaWordSize {
                $size
            }

            fn get_size(&self) -> usize {
                core::mem::size_of_val(self)
            }

            fn get_target_type(&self) -> DmaType {
                DmaType::Memory
            }
        }

        impl DmaSink for &mut $target {
            fn get_addr(&self) -> DmaAddress {
                self.as_ptr().into()
            }
        
            fn get_word_size(&self) -> DmaWordSize {
                $size
            }

            fn get_size(&self) -> usize {
                core::mem::size_of_val(self)
            }

            fn get_target_type(&self) -> DmaType {
                DmaType::Memory
            }
        })+
    }
}

dma_buffer!([u8], [i8] => DmaWordSize::Byte);
dma_buffer!([u16], [i16] => DmaWordSize::HalfWord);
dma_buffer!([u32], [usize], [i32], [isize] => DmaWordSize::Word);

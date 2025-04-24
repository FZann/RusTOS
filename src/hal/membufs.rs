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

use core::ptr::NonNull;

use crate::hal::dma::DmaStream;

/// Ring buffer to read data with DMA
pub struct DMARingBuffer<const SIZE: usize> {
	buff: [u8; SIZE],
    overflw: bool,
    read: usize,
    dma: NonNull<dyn DmaStream>,
}

impl<const SIZE: usize> DMARingBuffer<SIZE> {
    pub const fn new(dma: &'static dyn DmaStream) -> Self {
        Self {
            buff: [0; SIZE],
            overflw: false,
            read: 0,
            dma: unsafe { NonNull::new_unchecked(dma as *const dyn DmaStream as *mut dyn DmaStream) },
        }
    }
}

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

const DMA_ADR: usize = 0x4042_A000;
use crate::dma_buffer;

#[derive(Debug, Clone, Copy)]
pub enum DmaWordSize {
    Byte,
    HalfWord,
    Word,
    DoubleWord,
    QuadWord,
}


dma_buffer!(&[u8], &[i8] => DmaWordSize::Byte);
dma_buffer!(&[u16], &[i16] => DmaWordSize::HalfWord);
dma_buffer!(&[u32], &[usize], &[i32], &[isize] => DmaWordSize::Word);
dma_buffer!(&[u64], &[i64] => DmaWordSize::DoubleWord);
dma_buffer!(&[u128], &[i128] => DmaWordSize::QuadWord);

dma_buffer!(&mut [u8], &mut [i8] => DmaWordSize::Byte);
dma_buffer!(&mut [u16], &mut [i16] => DmaWordSize::HalfWord);
dma_buffer!(&mut [u32], &mut [usize], &mut [i32], &mut [isize] => DmaWordSize::Word);
dma_buffer!(&mut [u128], &mut [i128] => DmaWordSize::QuadWord);
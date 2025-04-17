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
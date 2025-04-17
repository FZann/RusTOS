use crate::hal::dma::DmaStream;

/// Ring buffer per data input con DMA connesso.
pub struct DMARingBuffer<const SIZE: usize> {
	buff: [u8; SIZE],
    overflw: bool,
    read: usize,
    dma: *mut dyn DmaStream,
}

impl<const SIZE: usize> DMARingBuffer<SIZE> {
    pub const fn new(dma: &'static dyn DmaStream) -> Self {
        Self {
            buff: [0; SIZE],
            overflw: false,
            read: 0,
            dma: dma as *const dyn DmaStream as *mut dyn DmaStream
        }
    }
}

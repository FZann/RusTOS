use crate::kernel::registers::*;
use crate::hal::dma::*;
use super::rcc::*;
use crate::dma_buffer;

const DMA1_ADR: usize = 0x4002_0000;
const DMA1_CH1_ADR: usize = DMA1_ADR + 0x008;

const DMA2_ADR: usize = 0x4002_0400;
const DMA2_CH1_ADR: usize = DMA2_ADR + 0x008;

const DMA_CH_OFFSET: usize = 0x14;

const CCR_EN_BIT: usize = 0;
const CCR_EN_MASK: usize = 1 << CCR_EN_BIT;

const CCR_TCIE_BIT: usize = 1;
const CCR_TCIE_MASK: usize = 1 << CCR_TCIE_BIT;

const CCR_HTIE_BIT: usize = 2;
const CCR_HTIE_MASK: usize = 1 << CCR_HTIE_BIT;

const CCR_TEIE_BIT: usize = 3;
const CCR_TEIE_MASK: usize = 1 << CCR_TEIE_BIT;

const CCR_DIR_BIT: usize = 4;
const CCR_DIR_MASK: usize = 1 << CCR_DIR_BIT;

const CCR_CIRC_BIT: usize = 5;
const CCR_CIRC_MASK: usize = 1 << CCR_CIRC_BIT;

const CCR_PINC_BIT: usize = 6;
const CCR_PINC_MASK: usize = 1 << CCR_PINC_BIT;

const CCR_MINC_BIT: usize = 7;
const CCR_MINC_MASK: usize = 1 << CCR_MINC_BIT;

const CCR_MEM2MEM_BIT: usize = 14;
const CCR_MEM2MEM_MASK: usize = 1 << CCR_MEM2MEM_BIT;

#[derive(Debug, Clone, Copy)]
pub enum DmaWordSize {
    Byte,
    HalfWord,
    Word,
}

struct DMA<const ADR: usize> {
    /// Interrupt status register
    isr: RO<ADR, 0x00>,

    /// Interrupt flag clear register
    ifcr: WO<ADR, 0x04>,
}

pub(crate) struct DmaChannel<const ADR: usize> {
    /// Channel control register
    ccr: RW<ADR, 0x00>,

    /// Channel number of data to transfer register
    cndtr: RW<ADR, 0x04>,

    /// Channel peripheral address register
    cpar: RW<ADR, 0x08>,

    /// Channel memory address register
    cmar: RW<ADR, 0x10>,

    void: RW<ADR, 0x14>,
}


impl Peripheral for DMA<DMA1_ADR> {
    type Registers = DMA<DMA1_ADR>;
    const ADR: usize = DMA1_ADR;
}

impl ClockEnable for DMA<DMA1_ADR> {
    const CLK_EN_BIT: usize = 0;
    const CLK_EN_BUS: RccBus = RccBus::AHB1;
}

impl Peripheral for DMA<DMA2_ADR> {
    type Registers = DMA<DMA2_ADR>;
    const ADR: usize = DMA2_ADR;
}

impl ClockEnable for DMA<DMA2_ADR> {
    const CLK_EN_BIT: usize = 1;
    const CLK_EN_BUS: RccBus = RccBus::AHB1;
}

impl DmaWordSize {
    fn mask(self) -> usize {
        match self {
            DmaWordSize::Byte => 0b00,
            DmaWordSize::HalfWord => 0b01,
            DmaWordSize::Word => 0b10,
        }
    }
}

impl DmaMode {
    fn mask(self) -> usize {
        match self {
            DmaMode::SingleWord => 0,
            DmaMode::SingleBlock => 0,
            DmaMode::RepeatedWord => CCR_CIRC_MASK,
            DmaMode::RepeatedBlock => CCR_CIRC_MASK,
        }
    }
}

impl DmaInterrupts {
    fn mask(self) -> usize {
        match self {
            DmaInterrupts::TRxComplete => CCR_TCIE_MASK,
            DmaInterrupts::HalfTRx => CCR_HTIE_MASK,
            DmaInterrupts::TRxError => CCR_TEIE_MASK,
        }
    }
}

macro_rules! make_dmachannels {
    (DMA $n: tt : $adr: expr => CH: [$($ch:expr),+]) => {
        paste::paste! {
            $(
                pub struct [<DMA $n CH $ch>];

                impl crate::kernel::registers::Peripheral for [<DMA $n CH $ch>] {
                    type Registers = DmaChannel<{ $adr + (DMA_CH_OFFSET * $ch) }>;
                    const ADR: usize = { $adr + (DMA_CH_OFFSET * $ch) };
                }

                impl DmaStream for [<DMA $n CH $ch>] {
                    fn ch_id(&self) -> u8 {
                        $ch as u8
                    }

                    fn enable_stream(&mut self) {
                        [<DMA $n CH $ch>]::regs().ccr.set_bit(CCR_EN_BIT);
                    }
                
                    fn disable_stream(&mut self) {
                        [<DMA $n CH $ch>]::regs().ccr.clear_bit(CCR_EN_BIT);
                    }
                
                    fn set_transfer(&mut self, src: &dyn DmaSource, dst: &dyn DmaSink, mode: DmaMode, size: usize) {
                        [<DMA $n CH $ch>]::regs().ccr.clear_bit(CCR_EN_BIT);
                        [<DMA $n CH $ch>]::regs().ccr.set(mode.mask());
                        [<DMA $n CH $ch>]::regs().cndtr.write(size);
                
                        match (src.get_target_type(), dst.get_target_type()) {
                            (DmaType::Peripheral, DmaType::Memory) => {
                                [<DMA $n CH $ch>]::regs().ccr.clear(CCR_DIR_MASK | CCR_PINC_MASK | CCR_MEM2MEM_MASK);
                                [<DMA $n CH $ch>]::regs().ccr.set_bit(CCR_MINC_MASK);
                
                                [<DMA $n CH $ch>]::regs().cpar.write(src.get_addr().into());
                                [<DMA $n CH $ch>]::regs().ccr.set(src.get_word_size().mask() << 10);
                
                                [<DMA $n CH $ch>]::regs().cmar.write(dst.get_addr().into());
                                [<DMA $n CH $ch>]::regs().ccr.set(dst.get_word_size().mask() << 8);
                            }
                
                            (DmaType::Peripheral, DmaType::Peripheral) => {
                                [<DMA $n CH $ch>]::regs().ccr.clear(CCR_DIR_MASK | CCR_MEM2MEM_MASK);
                
                                [<DMA $n CH $ch>]::regs().cpar.write(src.get_addr().into());
                                [<DMA $n CH $ch>]::regs().ccr.set(src.get_word_size().mask() << 10);
                
                                [<DMA $n CH $ch>]::regs().cmar.write(dst.get_addr().into());
                                [<DMA $n CH $ch>]::regs().ccr.set(dst.get_word_size().mask() << 8);
                            }
                
                            (DmaType::Memory, DmaType::Peripheral) => {
                                [<DMA $n CH $ch>]::regs().ccr.set(CCR_DIR_MASK | CCR_MINC_MASK);
                                [<DMA $n CH $ch>]::regs().ccr.clear(CCR_MEM2MEM_MASK | CCR_PINC_MASK);
                
                                [<DMA $n CH $ch>]::regs().cmar.write(src.get_addr().into());
                                [<DMA $n CH $ch>]::regs().ccr.set(src.get_word_size().mask() << 10);
                
                                [<DMA $n CH $ch>]::regs().cpar.write(dst.get_addr().into());
                                [<DMA $n CH $ch>]::regs().ccr.set(dst.get_word_size().mask() << 8);
                            }
                
                            (DmaType::Memory, DmaType::Memory) => {
                                [<DMA $n CH $ch>]::regs().ccr.clear(CCR_DIR_MASK);
                                [<DMA $n CH $ch>]::regs().ccr.set(CCR_MEM2MEM_MASK | CCR_MINC_MASK | CCR_PINC_MASK);
                
                                [<DMA $n CH $ch>]::regs().cpar.write(src.get_addr().into());
                                [<DMA $n CH $ch>]::regs().ccr.set(src.get_word_size().mask() << 10);
                
                                [<DMA $n CH $ch>]::regs().cmar.write(dst.get_addr().into());
                                [<DMA $n CH $ch>]::regs().ccr.set(dst.get_word_size().mask() << 8);
                            }
                        }
                    }
                
                    fn enable_interrupt(&mut self, irq: DmaInterrupts) {
                        [<DMA $n CH $ch>]::regs().ccr.set(irq.mask());
                    }
                    
                    fn disable_interrupt(&mut self, irq: DmaInterrupts) {
                        [<DMA $n CH $ch>]::regs().ccr.clear(irq.mask());
                    }
                }
            )+
        }
    }
}


make_dmachannels!(DMA 1: DMA1_CH1_ADR => CH: [0, 1, 2, 3, 4, 5, 6, 7]);
make_dmachannels!(DMA 2: DMA2_CH1_ADR => CH: [0, 1, 2, 3, 4, 5, 6, 7]);

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


use crate::kernel::registers::*;
use crate::hal::uart::*;
use crate::hal::dma::*;
use crate::hal::gpio::*;

use super::rcc::*;


//*********************************************************************************************************************
// DICHIARAZIONE VARIABILI CONNESSE ALL'HW
//*********************************************************************************************************************

//********************* INDIRIZZI *************************
const USART1_ADR: usize = 0x4001_3800;
const USART2_ADR: usize = 0x4000_4400;
const USART3_ADR: usize = 0x4000_4800;
const UART4_ADR: usize = 0x4000_4C00;
const UART5_ADR: usize = 0x4000_5000;

//********************* DMA TRIGGERS *************************
const USART1_RX_DMA_TRG: usize = 24;
const USART1_TX_DMA_TRG: usize = 25;
const USART2_RX_DMA_TRG: usize = 26;
const USART2_TX_DMA_TRG: usize = 27;
const USART3_RX_DMA_TRG: usize = 28;
const USART3_TX_DMA_TRG: usize = 29;
const UART4_RX_DMA_TRG: usize = 30;
const UART4_TX_DMA_TRG: usize = 31;
const UART5_RX_DMA_TRG: usize = 32;
const UART5_TX_DMA_TRG: usize = 33;

//********************* BIT MASKS *************************
const CR1_UEN: usize = 1;
const CR1_RE: usize = 1 << 2;
const CR1_TE: usize = 1 << 3;
const CR1_M0: usize = 1 << 12;
const CR1_DEAT: usize = 1 << 21;
const CR1_DEAT_POS: usize = 21;
const CR1_DEDT: usize = 1 << 25;
const CR1_DEDT_POS: usize = 25;
const CR1_M1: usize = 1 << 28;
const CR1_FIFOEN: usize = 1 << 29;

/// Half-duplex flag (solo pin TX - TRx tutta sullo stesso pin)
const CR2_HDSEL: usize = 1 << 3;
const CR2_MSBFIRST: usize = 1 << 19;

const CR3_DEM: usize = 1 << 14;
const CR3_DEP: usize = 1 << 15;

const ISR_RXFNE: usize = 1 << 5;
const ISR_TXFNF: usize = 1 << 7;
const ISR_TXFE: usize = 1 << 23;
const ISR_RXFF: usize = 1 << 24;


//*********************************************************************************************************************
// DICHIARAZIONE UART
//*********************************************************************************************************************

pub(crate) struct UART<const ADR: usize> {
    cr1: RW<ADR, 0x00>,
    cr2: RW<ADR, 0x04>,
    cr3: RW<ADR, 0x08>,

    /// Baud rate register
    brr: RW<ADR, 0x0C>,

    /// Guard time and prescaler register
    gtpr: RW<ADR, 0x10>,

    /// Receiver timeout register
    rtor: RW<ADR, 0x14>,

    /// Request register
    rqr: RW<ADR, 0x18>,

    /// Interrupt and status register
    isr: RW<ADR, 0x1C>,

    ///Interrupt clear register
    icr: RW<ADR, 0x20>,

    /// Read data register
    rdr: RO<ADR, 0x24>,

    /// Transmit data register
    tdr: RW<ADR, 0x28>,

    /// Prescaler register
    presc: RW<ADR, 0x2C>,
}

impl<const ADR: usize> UART<ADR> {
    const RDR: usize = ADR + 0x24;
    const TDR: usize = ADR + 0x28;

    #[inline]
    fn init_hw(&mut self) {
        //self.cr2.set(CR2_MSBFIRST);
        self.cr1.set(CR1_FIFOEN | CR1_TE | CR1_RE);
        
        self.cr1.set(CR1_UEN);
    }

    #[inline]
    fn deinit_hw(&mut self) {
        self.cr3.write(0);
        self.cr2.write(0);
        self.cr1.write(0);
    }

    #[inline]
    fn tx(&self, tx: u8) {
        self.tdr.write(tx as usize);
    }

    #[inline]
    fn rx(&self) -> u8 {
        self.rdr.read() as u8
    } 

    #[inline]
    fn rx_fifo_not_empty(&self) -> bool {
        self.isr.check(ISR_RXFNE)
    }

    #[inline]
    fn tx_fifo_not_full(&self) -> bool {
        self.isr.check(ISR_TXFNF)
    }

    #[inline]
    fn set_baud(&mut self, baud: usize) {
        let clock: crate::kernel::Hz = super::CPU_FREQUENCY.into();
        let clock: usize = clock.into();
        self.brr.write(clock / baud);
    }

    #[inline]
    fn set_mode(&mut self, mode: SerialMode) {
        match mode {
            SerialMode::Mode7N1 => self.cr1.modify(|r| r | CR1_M1 & !CR1_M0),
            SerialMode::Mode8N1 => self.cr1.clear(CR1_M1 | CR1_M0),
            SerialMode::Mode9N1 => self.cr1.modify(|r| r & !CR1_M1 | CR1_M0),
        }
    }

    #[inline]
    fn set_proto(&mut self, proto: SerialProto) {
        match proto {
            SerialProto::RS232 => {
                self.cr3.clear(CR3_DEP | CR3_DEM);
                self.cr2.clear(CR2_HDSEL);
            },
            SerialProto::RS485 => {
                // Mezzo carattere di assertion-time
                self.cr1.set(0x08 << CR1_DEAT_POS | 0x08 << CR1_DEDT_POS);
                self.cr2.set(CR2_HDSEL);
                self.cr3.modify(|r| r & !CR3_DEP | CR3_DEM);
            },
            SerialProto::LIN =>  { 
                self.cr3.clear(CR3_DEP | CR3_DEM);
                self.cr2.clear(CR2_HDSEL);
                todo!();
            },
        }
    }

    #[inline]
    fn set_msb(&mut self, typ: TRxType) {
        self.cr1.clear(CR1_UEN);
        match typ {
            TRxType::LsbFirst => self.cr1.clear(CR2_MSBFIRST),
            TRxType::MsbFirst => self.cr1.set(CR2_MSBFIRST),
        }
        self.cr1.set(CR1_UEN);
    }
}



macro_rules! make_uarts {
    ($peripheral:ident: $regs:ident, $addr:expr) => {
        pub struct $peripheral;

        impl crate::kernel::registers::Peripheral for $peripheral {
            type Registers = $regs<$addr>;
            const ADR: usize = $addr;
        }

        impl Uart for $peripheral {
            #[inline]
            fn tx(&self, tx: u8) {
                $peripheral::regs().tx(tx);
            }

            #[inline]
            fn rx(&self) -> u8 {
                $peripheral::regs().rx()
            }

            #[inline]
            fn rx_fifo_not_empty(&self) -> bool {
                $peripheral::regs().rx_fifo_not_empty()
            }

            #[inline]
            fn tx_fifo_not_full(&self) -> bool {
                $peripheral::regs().tx_fifo_not_full()
            }

            #[inline]
            fn init(baud: usize, mode: SerialMode, proto: SerialProto) -> Self {
                Self::activate_clock();
                let mut uart = Self {};
                uart.set_baud(baud);
                uart.set_mode(mode);
                uart.set_proto(proto);
                uart.activate();
                uart
            }

            #[inline]
            fn activate(&mut self) -> &mut Self {
                $peripheral::regs().init_hw();
                self
            }

            #[inline]
            fn deactivate(&mut self) -> &mut Self {
                $peripheral::regs().deinit_hw();
                self
            }
        
            #[inline]
            fn set_baud(&mut self, baud: usize) -> &mut Self {
                $peripheral::regs().set_baud(baud);
                self
            }

            #[inline]
            fn set_mode(&mut self, mode: SerialMode) -> &mut Self {
                $peripheral::regs().set_mode(mode);
                self
            }

            #[inline]
            fn set_proto(&mut self, proto: SerialProto) -> &mut Self {
                $peripheral::regs().set_proto(proto);
                self
            }

            #[inline]
            fn set_msb(&mut self, typ: TRxType) -> &mut Self {
                $peripheral::regs().set_msb(typ);
                self
            }

            fn set_tx_pin(pin: &mut impl PinSetup) {
                pin.init();
            }

            fn set_rx_pin(pin: &mut impl PinSetup) {
                pin.init();
            }
        }
    };
}


make_uarts!(UART1: UART, USART1_ADR);
make_uarts!(UART2: UART, USART2_ADR);
make_uarts!(UART3: UART, USART3_ADR);
make_uarts!(UART4: UART, UART4_ADR);
make_uarts!(UART5: UART, UART5_ADR);

impl UART1 {
    pub const TX1: PA9<Alternate<AF7>> = PA9::allocate();
    pub const RX1: PA10<Alternate<AF7>> = PA10::allocate();
    pub const TX2: PB6<Alternate<AF7>> = PB6::allocate();
    pub const RX2: PB7<Alternate<AF7>> = PB7::allocate();
    pub const TX3: PC4<Alternate<AF7>> = PC4::allocate();
    pub const RX3: PC5<Alternate<AF7>> = PC5::allocate();
    pub const TX4: PE0<Alternate<AF7>> = PE0::allocate();
    pub const RX4: PE1<Alternate<AF7>> = PE1::allocate();
}

impl UART2 {
}

impl UART3 {
}

impl UART4 {
    pub const TX1: PC10<Alternate<AF7>> = PC10::allocate();
    pub const RX1: PC11<Alternate<AF7>> = PC11::allocate();
}

impl UART5 {
}

impl ClockEnable for UART1 {
    const CLK_EN_BIT: usize = 14;
    const CLK_EN_BUS: RccBus = RccBus::APB2;
}

impl ClockEnable for UART2 {
    const CLK_EN_BIT: usize = 17;
    const CLK_EN_BUS: RccBus = RccBus::APB1_1;
}

impl ClockEnable for UART3 {
    const CLK_EN_BIT: usize = 18;
    const CLK_EN_BUS: RccBus = RccBus::APB1_1;
}

impl ClockEnable for UART4 {
    const CLK_EN_BIT: usize = 19;
    const CLK_EN_BUS: RccBus = RccBus::APB1_1;
}

impl ClockEnable for UART5 {
    const CLK_EN_BIT: usize = 20;
    const CLK_EN_BUS: RccBus = RccBus::APB1_1;
}

impl DmaPeripheralSource for UART1 {
    const SRC: DmaAddress = DmaAddress::new(Self::Registers::RDR);
    const MODE: DmaMode = DmaMode::RepeatedWord;
    const TRG: DmaTrigger = DmaTrigger::new(USART1_RX_DMA_TRG);
    const WORD: DmaWordSize = DmaWordSize::Byte;
}

impl DmaPeripheralSink for UART1 {
    const DST: DmaAddress = DmaAddress::new(Self::Registers::TDR);
    const MODE: DmaMode = DmaMode::RepeatedWord;
    const TRG: DmaTrigger = DmaTrigger::new(USART1_TX_DMA_TRG);
    const WORD: DmaWordSize = DmaWordSize::Byte;
}

impl DmaPeripheralSource for UART2 {
    const SRC: DmaAddress = DmaAddress::new(Self::Registers::RDR);
    const MODE: DmaMode = DmaMode::RepeatedWord;
    const TRG: DmaTrigger = DmaTrigger::new(USART2_RX_DMA_TRG);
    const WORD: DmaWordSize = DmaWordSize::Byte;
}

impl DmaPeripheralSink for UART2 {
    const DST: DmaAddress = DmaAddress::new(Self::Registers::TDR);
    const MODE: DmaMode = DmaMode::RepeatedWord;
    const TRG: DmaTrigger = DmaTrigger::new(USART2_TX_DMA_TRG);
    const WORD: DmaWordSize = DmaWordSize::Byte;
}

impl DmaPeripheralSource for UART3 {
    const SRC: DmaAddress = DmaAddress::new(Self::Registers::RDR);
    const MODE: DmaMode = DmaMode::RepeatedWord;
    const TRG: DmaTrigger = DmaTrigger::new(USART3_RX_DMA_TRG);
    const WORD: DmaWordSize = DmaWordSize::Byte;
}

impl DmaPeripheralSink for UART3 {
    const DST: DmaAddress = DmaAddress::new(Self::Registers::TDR);
    const MODE: DmaMode = DmaMode::RepeatedWord;
    const TRG: DmaTrigger = DmaTrigger::new(USART3_TX_DMA_TRG);
    const WORD: DmaWordSize = DmaWordSize::Byte;
}

impl DmaPeripheralSource for UART4 {
    const SRC: DmaAddress = DmaAddress::new(Self::Registers::RDR);
    const MODE: DmaMode = DmaMode::RepeatedWord;
    const TRG: DmaTrigger = DmaTrigger::new(UART4_RX_DMA_TRG);
    const WORD: DmaWordSize = DmaWordSize::Byte;
}

impl DmaPeripheralSink for UART4 {
    const DST: DmaAddress = DmaAddress::new(Self::Registers::TDR);
    const MODE: DmaMode = DmaMode::RepeatedWord;
    const TRG: DmaTrigger = DmaTrigger::new(UART4_TX_DMA_TRG);
    const WORD: DmaWordSize = DmaWordSize::Byte;
}

impl DmaPeripheralSource for UART5 {
    const SRC: DmaAddress = DmaAddress::new(Self::Registers::RDR);
    const MODE: DmaMode = DmaMode::RepeatedWord;
    const TRG: DmaTrigger = DmaTrigger::new(UART5_RX_DMA_TRG);
    const WORD: DmaWordSize = DmaWordSize::Byte;
}

impl DmaPeripheralSink for UART5 {
    const DST: DmaAddress = DmaAddress::new(Self::Registers::TDR);
    const MODE: DmaMode = DmaMode::RepeatedWord;
    const TRG: DmaTrigger = DmaTrigger::new(UART5_TX_DMA_TRG);
    const WORD: DmaWordSize = DmaWordSize::Byte;
}
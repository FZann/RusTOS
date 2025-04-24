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


use crate::kernel::{registers::*, Kernel};

#[cfg(feature = "clock_out")]
use crate::hal::gpio::*;

//*********************************************************************************************************************
// RCC ENUMERATIONS
//*********************************************************************************************************************

#[derive(Clone, Copy, Debug)]
pub enum RccBus {
    AHB1,
    AHB2,
    AHB3,
    APB1_1,
    APB1_2,
    APB2,
}

pub trait ClockEnable: Peripheral + Sized {
    const CLK_EN_BIT: usize;
    const CLK_EN_BUS: RccBus;

    fn activate_clock() {
        RCC::regs().enable_clock::<Self>();
    }
}

#[derive(Clone, Copy, Debug)]
pub enum McoPresc {
    DIV1,
    DIV2,
    DIV4,
    DIV8,
    DIV16,
}

impl McoPresc {
    const CFRG_MCOPRE_POS: usize = 28;

    #[inline]
    const fn value(self) -> usize {
        match self {
            McoPresc::DIV1 => 0b000,
            McoPresc::DIV2 => 0b001,
            McoPresc::DIV4 => 0b010,
            McoPresc::DIV8 => 0b011,
            McoPresc::DIV16 => 0b100,
        }
    }

    #[inline]
    const fn mask(self) -> usize {
        self.value() << Self::CFRG_MCOPRE_POS
    }
}

#[derive(Clone, Copy, Debug)]
pub enum McoOutput {
    SYSCLK,
    HSI16,
    HSE,
    PLL,
    LSI,
    LSE,
    HSI48,
}

impl McoOutput {
    const CFRG_MCOSEL_POS: usize = 24;

    #[inline]
    const fn value(self) -> usize {
        match self {
            McoOutput::SYSCLK => 0b0001,
            McoOutput::HSI16 => 0b0011,
            McoOutput::HSE => 0b0100,
            McoOutput::PLL => 0b0101,
            McoOutput::LSI => 0b0110,
            McoOutput::LSE => 0b0111,
            McoOutput::HSI48 => 0b1000,
        }
    }

    #[inline]
    const fn mask(self) -> usize {
        self.value() << Self::CFRG_MCOSEL_POS
    }
}


//*********************************************************************************************************************
// HW-CONNECTED VARIABLES
//*********************************************************************************************************************

//********************* ADDRESSES *************************
const RCC_ADDR: usize = 0x4002_1000;


//********************* BIT MASKS *************************
const CFGR_SW_PLL_MASK: usize = 0b11;


#[derive(Clone, Copy, Debug)]
enum PllSource {
    HSI16,
    HSE,
}

impl PllSource {
    const PLLCFGR_PLLSRC_POS: usize = 0;

    const fn value(self) -> usize {
        let value = match self {
            PllSource::HSI16 => 0b10,
            PllSource::HSE => 0b11,
        };

        value << Self::PLLCFGR_PLLSRC_POS
    }
}



#[derive(Clone, Copy, Debug)]
enum PllDiv {
    Div1,
    Div2,
    Div3,
    Div4,
    Div5,
    Div6,
    Div7,
    Div8,
    Div9,
    Div10,
    Div11,
    Div12,
    Div13,
    Div14,
    Div15,
    Div16,
}

impl PllDiv {
    const PLLCFGR_PLLM_POS: usize = 4;

    const fn value(self) -> usize {
        let value = match self {
            PllDiv::Div1 => 0,
            PllDiv::Div2 => 1,
            PllDiv::Div3 => 2,
            PllDiv::Div4 => 3,
            PllDiv::Div5 => 4,
            PllDiv::Div6 => 5,
            PllDiv::Div7 => 6,
            PllDiv::Div8 => 7,
            PllDiv::Div9 => 8,
            PllDiv::Div10 => 9,
            PllDiv::Div11 => 10,
            PllDiv::Div12 => 11,
            PllDiv::Div13 => 12,
            PllDiv::Div14 => 13,
            PllDiv::Div15 => 14,
            PllDiv::Div16 => 15,
        };

        value << Self::PLLCFGR_PLLM_POS
    }
}



#[derive(Clone, Copy, Debug)]
enum PllMul {
    Mul50,
    Mul75,
    Mul85
}

impl PllMul {
    const PLLCFGR_PLLN_POS: usize = 8;

    const fn value(self) -> usize {
        let value = match self {
            PllMul::Mul50 => 50,
            PllMul::Mul75 => 75,
            PllMul::Mul85 => 85,
        };

        value << Self::PLLCFGR_PLLN_POS
    }
}


#[derive(Clone, Copy, Debug)]
enum AhbDiv {
    Div1,
    Div2,
    Div4,
    Div8,
    Div16,
    Div64,
    Div128,
    Div256,
    Div512,
}

impl AhbDiv {
    const CFGR_HPRE_POS: usize = 4;

    const fn value(self) -> usize {
        let value = match self {
            AhbDiv::Div1 => 0b0000,
            AhbDiv::Div2 => 0b1000,
            AhbDiv::Div4 => 0b1001,
            AhbDiv::Div8 => 0b1010,
            AhbDiv::Div16 => 0b1011,
            AhbDiv::Div64 => 0b1100,
            AhbDiv::Div128 => 0b1101,
            AhbDiv::Div256 => 0b1110,
            AhbDiv::Div512 => 0b1111,
        };

        value << Self::CFGR_HPRE_POS
    }

    #[inline]
    const fn mask() -> usize {
        0b1111 << Self::CFGR_HPRE_POS
    }
}

enum SysClockSwitch {
    HSI16,
    HSE,
    PLL
}

impl SysClockSwitch {
    const CFGR_SW_POS: usize = 0;

    const fn value(self) -> usize {
        let value = match self {
            SysClockSwitch::HSI16 => 0b01,
            SysClockSwitch::HSE => 0b10,
            SysClockSwitch::PLL => 0b11,
        };

        value << Self::CFGR_SW_POS
    }

    #[inline]
    const fn mask() -> usize {
        0b11 << Self::CFGR_SW_POS
    }
}

//*********************************************************************************************************************
// RCC DECLARATION
//*********************************************************************************************************************
pub struct RCC {
    cr: RW<RCC_ADDR, 0>,
    icscr: RW<RCC_ADDR,0x04>,
    cfgr: RW<RCC_ADDR,0x08>,
    pllcfgr: RW<RCC_ADDR,0x0C>,
    cier: RW<RCC_ADDR,0x18>,
    cifr: RW<RCC_ADDR,0x1C>,
    cicr: RW<RCC_ADDR,0x20>,
    ahb1rstr: RW<RCC_ADDR,0x28>,
    ahb2rstr: RW<RCC_ADDR,0x2C>,
    ahb3rstr: RW<RCC_ADDR,0x30>,
    apb1rstr1: RW<RCC_ADDR,0x38>,
    apb1rstr2: RW<RCC_ADDR,0x3C>,
    apb2rstr: RW<RCC_ADDR,0x40>,
    ahb1enr: RW<RCC_ADDR,0x48>,
    ahb2enr: RW<RCC_ADDR,0x4C>,
    ahb3enr: RW<RCC_ADDR,0x50>,
    apb1enr1: RW<RCC_ADDR,0x58>,
    apb1enr2: RW<RCC_ADDR,0x5C>,
    apb2enr: RW<RCC_ADDR,0x60>,
}

impl Peripheral for RCC {
    type Registers = RCC;
    const ADR: usize = RCC_ADDR;
}

impl RCC {
    const CR_PLLON_BIT: usize = 24;
    const CR_PLLSYSRDY_BIT: usize = 25;
    const PLLCFGR_PLLPEN_BIT: usize = 16;
    const PLLCFGR_PLLREN_BIT: usize = 24;

    // Reference Manual, 6.2.5:
    // The device embeds 3 PLLs: PLL, PLLSAI1, PLLSAI2. Each PLL provides up to three
    // independent outputs. The internal PLLs can be used to multiply the HSI16, HSE or MSI
    // output clock frequency. The PLLs input frequency must be between 4 and 16 MHz. The
    // selected clock source is divided by a programmable factor PLLM from 1 to 8 to provide a
    // clock frequency in the requested input range. Refer to Figure 15: Clock tree (for
    // STM32L47x/L48x devices) and Figure 16: Clock tree (for STM32L49x/L4Ax devices) and
    // PLL configuration register (RCC_PLLCFGR).
    // The PLLs configuration (selection of the input clock and multiplication factor) must be done
    // before enabling the PLL. Once the PLL is enabled, these parameters cannot be changed.
    // To modify the PLL configuration, proceed as follows:
    // 1. Disable the PLL by setting PLLON to 0 in Clock control register (RCC_CR).
    // 2. Wait until PLLRDY is cleared. The PLL is now fully stopped.
    // 3. Change the desired parameter.
    // 4. Enable the PLL again by setting PLLON to 1.
    // 5. Enable the desired PLL outputs by configuring PLLPEN, PLLQEN, PLLREN in PLL
    // configuration register (RCC_PLLCFGR).
    pub fn set_pll(&self, cpu_freq: crate::kernel::MHz) {
        self.cr.clear_bit(Self::CR_PLLON_BIT);
        while self.cr.read_bit(Self::CR_PLLSYSRDY_BIT) { }

        // Blank pllcfgr register and then set it up as wanted
        self.pllcfgr.write(0);
        self.pllcfgr.set(PllSource::HSI16.value());
        self.pllcfgr.set(PllDiv::Div4.value());
        self.pllcfgr.set(PllMul::Mul50.value());
        
        self.cr.set_bit(Self::CR_PLLON_BIT);
        while !self.cr.read_bit(Self::CR_PLLSYSRDY_BIT) { }

        // Sets a Div2 on AHB to give time (> 1us) to adapt to new clock
        self.cfgr.set(AhbDiv::Div2.value());
        self.cfgr.set(SysClockSwitch::PLL.value());

        self.pllcfgr.set_bit(Self::PLLCFGR_PLLREN_BIT);

        // Is this really 1us? Who knows...
        let mut delay = 0usize;
        loop {
            delay += 1;
            if delay >= 500000 {
                break;
            }
        }

        // Sets Div1 (NoDiv): as that value is all zeroes, we simply clear all bits of corresponding mask
        self.cfgr.clear(AhbDiv::mask());
    }


    pub fn enable_clock<P: ClockEnable>(&self) {
        match P::CLK_EN_BUS {
            RccBus::AHB1 => self.ahb1enr.set_bit(P::CLK_EN_BIT),
            RccBus::AHB2 => self.ahb2enr.set_bit(P::CLK_EN_BIT),
            RccBus::AHB3 => self.ahb3enr.set_bit(P::CLK_EN_BIT),
            RccBus::APB1_1 => self.apb1enr1.set_bit(P::CLK_EN_BIT),
            RccBus::APB1_2 => self.apb1enr2.set_bit(P::CLK_EN_BIT),
            RccBus::APB2 => self.apb2enr.set_bit(P::CLK_EN_BIT),
        }        
    }

    pub fn disable_clock<P: ClockEnable>(&self) {
        match P::CLK_EN_BUS {
            RccBus::AHB1 => self.ahb1enr.clear_bit(P::CLK_EN_BIT),
            RccBus::AHB2 => self.ahb2enr.clear_bit(P::CLK_EN_BIT),
            RccBus::AHB3 => self.ahb3enr.clear_bit(P::CLK_EN_BIT),
            RccBus::APB1_1 => self.apb1enr1.clear_bit(P::CLK_EN_BIT),
            RccBus::APB1_2 => self.apb1enr2.clear_bit(P::CLK_EN_BIT),
            RccBus::APB2 => self.apb2enr.clear_bit(P::CLK_EN_BIT),
        }        
    }

    pub fn set_mco(&self, presc: McoPresc, output: McoOutput) {
        self.cfgr.set(presc.mask());
        self.cfgr.set(output.mask());
    }

}

impl Kernel {
    #[inline]
    pub(crate) fn setup_clock(&self) {
        let rcc = RCC::regs();
        rcc.set_pll(super::CPU_FREQUENCY);
        rcc.set_mco(McoPresc::DIV16, McoOutput::SYSCLK);

        #[cfg(feature = "clock_out")]
        {
            let mut pa8: PA8<Alternate<AF0>> = PA8::allocate();
            pa8.init();
        }
    }
}
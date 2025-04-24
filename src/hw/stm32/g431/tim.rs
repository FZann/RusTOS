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
use crate::hal::tim::*;

use super::{rcc::*, CPU_FREQUENCY};


//*********************************************************************************************************************
// HW-CONNECTED VARIABLES
//*********************************************************************************************************************

//********************* ADDRESSES *************************
const TIM1_ADR: usize = 0x4001_2C00;
const TIM2_ADR: usize = 0x4000_0000;
const TIM3_ADR: usize = 0x4000_0400;
const TIM4_ADR: usize = 0x4000_0800;
const TIM5_ADR: usize = 0x4000_0C00;
const TIM6_ADR: usize = 0x4000_1000;
const TIM7_ADR: usize = 0x4000_1400;
const TIM8_ADR: usize = 0x4001_3400;
const TIM15_ADR: usize = 0x4001_4000;
const TIM16_ADR: usize = 0x4001_4400;
const TIM17_ADR: usize = 0x4001_4800;
const TIM20_ADR: usize = 0x4001_5000;


//********************* BIT MASKS *************************
const CR1_CEN: usize = 1 << 0;
const CR1_UDIS: usize = 1 << 1;
const CR1_URS: usize = 1 << 2;
const CR1_OPM: usize = 1 << 3;
const CR1_ARPE: usize = 1 << 7;
const CR1_UIFREMAP: usize = 1 << 11;
const CR1_DITHEN: usize = 1 << 12;

const CR2_CCPC: usize = 1 << 0;
const CR2_CCUS: usize = 1 << 2;
const CR2_CCDS: usize = 1 << 3;
const CR2_MMS: usize = 1 << 4;
const CR2_TI1S: usize = 1 << 7;
const CR2_OIS1: usize = 1 << 8;
const CR2_OISN1: usize = 1 << 9;
const CR2_OIS2: usize = 1 << 10;
const CR2_OISN2: usize = 1 << 11;
const CR2_OIS3: usize = 1 << 12;
const CR2_OISN3: usize = 1 << 13;
const CR2_OIS4: usize = 1 << 14;
const CR2_OISN4: usize = 1 << 15;
const CR2_OIS5: usize = 1 << 16;
const CR2_OIS6: usize = 1 << 18;

const SMCR_SMS: usize = 1 << 0;
const SMCR_OCCS: usize = 1 << 3;
const SMCR_TS: usize = 1 << 4;
const SMCR_MSM: usize = 1 << 7;
const SMCR_ETF: usize = 1 << 8;
const SMCR_ETPS: usize = 1 << 12;
const SMCR_ECE: usize = 1 << 14;
const SMCR_ETP: usize = 1 << 15;
const SMCR_SMS_2: usize = 1 << 16;
const SMCR_TS_2: usize = 1 << 20;
const SMCR_SMSPE: usize = 1 << 24;
const SMCR_SMSPS: usize = 1 << 25;

const DIER_UIE: usize = 1 << 0;
const DIER_CC1IE: usize = 1 << 1;
const DIER_CC2IE: usize = 1 << 2;
const DIER_CC3IE: usize = 1 << 3;
const DIER_CC4IE: usize = 1 << 4;
const DIER_COMIE: usize = 1 << 5;
const DIER_TIE: usize = 1 << 6;
const DIER_BIE: usize = 1 << 7;
const DIER_UDE: usize = 1 << 8;
const DIER_CC1DE: usize = 1 << 9;
const DIER_CC2DE: usize = 1 << 10;
const DIER_CC3DE: usize = 1 << 11;
const DIER_CC4DE: usize = 1 << 12;
const DIER_COMDE: usize = 1 << 13;
const DIER_TDE: usize = 1 << 14;
const DIER_IDXIE: usize = 1 << 20;
const DIER_DIRIE: usize = 1 << 21;
const DIER_IERRIE: usize = 1 << 22;
const DIER_TERRIE: usize = 1 << 23;

const SR_UIF: usize = 1 << 0;
const SR_CC1IF: usize = 1 << 1;
const SR_CC2IF: usize = 1 << 2;
const SR_CC3IF: usize = 1 << 3;
const SR_CC4IF: usize = 1 << 4;
const SR_COMIF: usize = 1 << 5;
const SR_TIF: usize = 1 << 6;
const SR_BIF: usize = 1 << 7;
const SR_B2IF: usize = 1 << 8;
const SR_CC1OF: usize = 1 << 9;
const SR_CC2OF: usize = 1 << 10;
const SR_CC3OF: usize = 1 << 11;
const SR_CC4OF: usize = 1 << 12;
const SR_COMDF: usize = 1 << 13;
const SR_SBIF: usize = 1 << 14;
const SR_CC5IF: usize = 1 << 16;
const SR_CC6IF: usize = 1 << 17;
const SR_IDXF: usize = 1 << 20;
const SR_DIRF: usize = 1 << 21;
const SR_IERRF: usize = 1 << 22;
const SR_TERRF: usize = 1 << 23;

const EGR_UG: usize = 1 << 0;
const EGR_CC1G: usize = 1 << 1;
const EGR_CC2G: usize = 1 << 2;
const EGR_CC3G: usize = 1 << 3;
const EGR_CC4G: usize = 1 << 4;
const EGR_COMG: usize = 1 << 5;
const EGR_TG: usize = 1 << 6;
const EGR_BG: usize = 1 << 7;
const EGR_B2G: usize = 1 << 8;

// Bit Masks for INPUT CAPTURE
const CCMR_CC1S_MASK: usize = 0b11;
const CCMR_CC2S_MASK: usize = 0b11_0000_0000;
const CCMR1_CC1S: usize = 1 << 0;
const CCMR1_IC1PSC: usize = 1 << 2;
const CCMR1_IC1F: usize = 1 << 4;
const CCMR1_CC2S: usize = 1 << 8;
const CCMR1_IC2PSC: usize = 1 << 10;
const CCMR1_IC2F: usize = 1 << 12;

const CCMR2_CC3S: usize = 1 << 0;
const CCMR2_IC3PSC: usize = 1 << 2;
const CCMR2_IC3F: usize = 1 << 4;
const CCMR2_CC4S: usize = 1 << 8;
const CCMR2_IC4PSC: usize = 1 << 10;
const CCMR2_IC4F: usize = 1 << 12;

// Bit Masks for OUTPUT COMPARE
const CCMR_OC1M_MASK: usize = 0x00010070;
const CCMR_OC1M_SHIFT: usize = 4;
const CCMR_OC2M_MASK: usize = 0x01007000;
const CCMR_OC2M_SHIFT: usize = 12;
//const CCMR1_CC1S: usize = 1 << 0;     // Same bits as IC
const CCMR1_OC1FE: usize = 1 << 2;
const CCMR1_OC1PE: usize = 1 << 3;
const CCMR1_OC1M: usize = 1 << 4;
const CCMR1_OC1CE: usize = 1 << 7;
//const CCMR1_CC2S: usize = 1 << 8;     // Same bits as IC
const CCMR1_OC2FE: usize = 1 << 10;
const CCMR1_OC2PE: usize = 1 << 11;
const CCMR1_OC2M: usize = 1 << 12;
const CCMR1_OC2CE: usize = 1 << 15;
const CCMR1_OC1M_2: usize = 1 << 16;
const CCMR1_OC2M_2: usize = 1 << 24;

//const CCMR1_CC1S: usize = 1 << 0;     // Same bits as IC
const CCMR2_OC3FE: usize = 1 << 2;
const CCMR2_OC3PE: usize = 1 << 3;
const CCMR2_OC3M: usize = 1 << 4;
const CCMR2_OC3CE: usize = 1 << 7;
//const CCMR1_CC2S: usize = 1 << 8;     // Same bits as IC
const CCMR2_OC4FE: usize = 1 << 10;
const CCMR2_OC4PE: usize = 1 << 11;
const CCMR2_OC4M: usize = 1 << 12;
const CCMR2_OC4CE: usize = 1 << 15;
const CCMR2_OC3M_2: usize = 1 << 16;
const CCMR2_OC4M_2: usize = 1 << 24;

const CCMR3_OC5FE: usize = 1 << 2;
const CCMR3_OC5PE: usize = 1 << 3;
const CCMR3_OC5M: usize = 1 << 4;
const CCMR3_OC6CE: usize = 1 << 7;
const CCMR3_OC6FE: usize = 1 << 10;
const CCMR3_OC6PE: usize = 1 << 11;
const CCMR3_OC6M: usize = 1 << 12;
const CCMR3_OC4CE: usize = 1 << 15;
const CCMR3_OC5M_2: usize = 1 << 16;
const CCMR3_OC6M_2: usize = 1 << 24;

const DTR2_DTGF: usize = 1 << 0;
const DTR2_DTAE: usize = 1 << 16;
const DTR2_DTPE: usize = 1 << 17;

const ECR_IE: usize = 1 << 0;
const ECR_IDIR: usize = 1 << 1;
const ECR_FIDX: usize = 1 << 5;
const ECR_IPOS: usize = 1 << 6;
const ECR_PW: usize = 1 << 16;
const ECR_PWPRSC: usize = 1 << 24;

const TISEL_TI1SEL: usize = 1 << 0;
const TISEL_TI2SEL: usize = 1 << 8;
const TISEL_TI3SEL: usize = 1 << 16;
const TISEL_TI4SEL: usize = 1 << 24;

const DCR_DBA: usize = 1 << 0;
const DCR_DBL: usize = 1 << 8;

const CCER_CC1_SHIFT: usize = 0;
const CCER_CC1N_SHIFT: usize = 2;
const CCER_CC1_MASK: usize = 0b1111 << CCER_CC1_SHIFT;

const CCER_CC2_SHIFT: usize = 4;
const CCER_CC2N_SHIFT: usize = 6;
const CCER_CC2_MASK: usize = 0b1111 << CCER_CC2_SHIFT;

const CCER_CC3_SHIFT: usize = 8;
const CCER_CC3N_SHIFT: usize = 10;
const CCER_CC3_MASK: usize = 0b1111 << CCER_CC3_SHIFT;

const CCER_CC4_SHIFT: usize = 12;
const CCER_CC4N_SHIFT: usize = 14;
const CCER_CC4_MASK: usize = 0b1111 << CCER_CC4_SHIFT;

const CCER_CC5_SHIFT: usize = 16;
const CCER_CC5_MASK: usize = 0b11 << CCER_CC5_SHIFT;

const CCER_CC6_SHIFT: usize = 20;
const CCER_CC6_MASK: usize = 0b11 << CCER_CC6_SHIFT;

//*********************************************************************************************************************
// MACRO FOR TIMER CREATION
//*********************************************************************************************************************
macro_rules! make_timebase {
    ($TIM:ident: $regs:ident, $addr:expr, $bits:ty, $dir:expr) => {
        pub struct $TIM;

        impl crate::kernel::registers::Peripheral for $TIM {
            type Registers = $regs<$addr>;
            const ADR: usize = $addr;
        }

        impl TimeBase for $TIM {
            type BITS = $bits;
            const DIR: CountDir = $dir;

            #[inline]
            fn init_clock(&self) {
                $TIM::activate_clock();
            }

            #[inline]
            fn start(&mut self) {
                $TIM::regs().cr1.set(CR1_ARPE | CR1_CEN);
            }

            #[inline]
            fn stop(&mut self) {
                $TIM::regs().cr1.clear(CR1_ARPE | CR1_CEN);
            }

            #[inline]
            fn set_mode(&mut self, mode: TimMode) {
                match mode {
                    TimMode::Loop => $TIM::regs().cr1.clear(CR1_OPM),
                    TimMode::OneShot => $TIM::regs().cr1.set(CR1_OPM),
                }
            }

            #[inline]
            fn set_frequency(&mut self, freq: Hz) {
                let cpu_f: Hz = CPU_FREQUENCY.into();
                let mut f: Hz = CPU_FREQUENCY.into();
                let mut presc = 1u32;
                let mut arr = f / freq;

                while (arr) > Self::BITS::MAX.into() {
                    presc += 1;
                    f = cpu_f / presc.into();
                    arr = f / freq;
                }

                // -1 arriva dal DS del G4:
                // psc == 0 => f / 1
                // psc == 1 => f / 2
                // psc == 2 => f / 3
                // etc....
                $TIM::regs().psc.write((presc - 1) as usize);
                $TIM::regs().arr.write(arr.into());
                $TIM::regs().cnt.write(0);
            }

            #[inline]
            fn activate_interrupt(&mut self) {
                $TIM::regs().sr.clear(SR_UIF);
                $TIM::regs().dier.set(DIER_UIE);
            }

            #[inline]
            fn deactivate_interrupt(&mut self) {
                $TIM::regs().sr.clear(SR_UIF);
                $TIM::regs().dier.clear(DIER_UIE);
            }
        }
    }
}

macro_rules! make_capturecompare {
    ($TIM:ident => CC: 1) => {
        impl OutputCompare<1> for $TIM {
            fn set_compare_value(&mut self, comp: Self::BITS) {
                $TIM::regs().ccr1.write(comp as usize);
            }
            
            fn set_compare_mode(&mut self, mode: CompareMode) {
                $TIM::regs().oc_ccmr1.clear(CCMR_CC1S_MASK);
                $TIM::regs().oc_ccmr1.clear(CCMR_OC1M_MASK);
                $TIM::regs().oc_ccmr1.set(mode.ocm1());
                $TIM::regs().ccer.clear(CCER_CC1_MASK);
                $TIM::regs().ccer.set(mode.cc1());
            }
        }

        impl InputCapture<1> for $TIM {

        }
    };

    ($TIM:ident => CC: 2) => {
        impl OutputCompare<2> for $TIM {
            fn set_compare_value(&mut self, comp: Self::BITS) {
                $TIM::regs().ccr2.write(comp as usize);
            }
            
            fn set_compare_mode(&mut self, mode: CompareMode) {
                $TIM::regs().oc_ccmr1.clear(CCMR_CC2S_MASK);
                $TIM::regs().oc_ccmr1.clear(CCMR_OC2M_MASK);
                $TIM::regs().oc_ccmr1.set(mode.ocm2());
                $TIM::regs().ccer.clear(CCER_CC2_MASK);
                $TIM::regs().ccer.set(mode.cc2());
            }
        }

        impl InputCapture<2> for $TIM {

        }
    };

    ($TIM:ident => CC: 3) => {
        impl OutputCompare<3> for $TIM {
            fn set_compare_value(&mut self, comp: Self::BITS) {
                $TIM::regs().ccr3.write(comp as usize);
            }
            
            fn set_compare_mode(&mut self, mode: CompareMode) {
                $TIM::regs().oc_ccmr2.clear(CCMR_CC1S_MASK);
                $TIM::regs().oc_ccmr2.clear(CCMR_OC1M_MASK);
                $TIM::regs().oc_ccmr2.set(mode.ocm1());
                $TIM::regs().ccer.clear(CCER_CC3_MASK);
                $TIM::regs().ccer.set(mode.cc3());
            }
        }

        impl InputCapture<3> for $TIM {

        }
    };

    ($TIM:ident => CC: 4) => {
        impl OutputCompare<4> for $TIM {
            fn set_compare_value(&mut self, comp: Self::BITS) {
                $TIM::regs().ccr4.write(comp as usize);
            }

            fn set_compare_mode(&mut self, mode: CompareMode) {
                $TIM::regs().oc_ccmr2.clear(CCMR_CC2S_MASK);
                $TIM::regs().oc_ccmr2.clear(CCMR_OC2M_MASK);
                $TIM::regs().oc_ccmr2.set(mode.ocm2());
                $TIM::regs().ccer.clear(CCER_CC4_MASK);
                $TIM::regs().ccer.set(mode.cc4());
            }
        }

        impl InputCapture<4> for $TIM {

        }
    };

    ($TIM:ident => OC: 5) => {
        impl OutputCompare<5> for $TIM {
            fn set_compare_value(&mut self, comp: Self::BITS) {
                $TIM::regs().ccr5.write(comp as usize);
            }

            fn set_compare_mode(&mut self, mode: CompareMode) {
                $TIM::regs().oc_ccmr3.clear(CCMR_OC1M_MASK);
                $TIM::regs().oc_ccmr3.set(mode.ocm1()); 
                $TIM::regs().ccer.clear(CCER_CC5_MASK);
                $TIM::regs().ccer.set(mode.cc5());
            }
        }
    };

    ($TIM:ident => OC: 6) => {
        impl OutputCompare<6> for $TIM {
            fn set_compare_value(&mut self, comp: Self::BITS) {
                $TIM::regs().ccr6.write(comp as usize);
            }

            fn set_compare_mode(&mut self, mode: CompareMode) {
                $TIM::regs().oc_ccmr3.clear(CCMR_OC2M_MASK);
                $TIM::regs().oc_ccmr3.set(mode.ocm2());
                $TIM::regs().ccer.clear(CCER_CC6_MASK);
                $TIM::regs().ccer.set(mode.cc6());
            }
        }
    };
}

//*********************************************************************************************************************
// TIMER ADDITIONAL FUNCTIONALITIES
//*********************************************************************************************************************
#[derive(Debug, Clone, Copy)]
pub enum CompareMode {
    /// Compare disable. Only timebase
    Frozen,   
    /// High pin on match              
    High,             
    /// Low pin on match      
    Low,                   
    /// Toggle pin on match
    Toggle,
    /// Always high
    ForcedHigh,
    /// Always low 
    ForcedLow,
    /// PWM start high, goes low on match
    PwmHighToLow,
    /// PWM start low, goes high on match
    PwmLowToHigh,
    /// Combined compare of 2 registers
    CombinedPwmHighToLow,
    /// Combined compare of 2 registers
    CombinedPwmLowToHigh,
    /// Uses 2 compare register on count up-down
    AsymmetricPwmHighToLow,
    /// Uses 2 compare register on count up-down
    AsymmetricPwmLowToHigh,
}

impl CompareMode {
    fn ocm(self) -> usize {
        match self {
            CompareMode::Frozen =>  0b0000000000000,
            CompareMode::High =>    0b0000000000001,
            CompareMode::Low =>     0b0000000000010,
            CompareMode::Toggle =>  0b0000000000011,
            CompareMode::ForcedHigh =>   0b0000000000101,
            CompareMode::ForcedLow =>    0b0000000000100,
            CompareMode::PwmHighToLow => 0b0000000000110,
            CompareMode::PwmLowToHigh => 0b0000000000111,
            CompareMode::CombinedPwmHighToLow =>   0b1000000000100,
            CompareMode::CombinedPwmLowToHigh =>   0b1000000000101,
            CompareMode::AsymmetricPwmHighToLow => 0b1000000000110,
            CompareMode::AsymmetricPwmLowToHigh => 0b1000000000111,
        }
    }

    fn ccer(self) -> usize {
        match self {
            CompareMode::Frozen => 0b00,
            _ => 0b01,
        }
    }

    fn ocm1(self) -> usize {
        self.ocm() << CCMR_OC1M_SHIFT
    }

    fn ocm2(self) -> usize {
        self.ocm() << CCMR_OC2M_SHIFT
    }

    fn cc1(self) -> usize {
        self.ccer() << CCER_CC1_SHIFT
    }

    fn cc2(self) -> usize {
        self.ccer() << CCER_CC2_SHIFT
    }

    fn cc3(self) -> usize {
        self.ccer() << CCER_CC3_SHIFT
    }

    fn cc4(self) -> usize {
        self.ccer() << CCER_CC4_SHIFT
    }

    fn cc5(self) -> usize {
        self.ccer() << CCER_CC5_SHIFT
    }

    fn cc6(self) -> usize {
        self.ccer() << CCER_CC6_SHIFT
    }

}


pub enum ComplementaryMode {
    Disable,
}

impl ComplementaryMode {

}

//*********************************************************************************************************************
// TIMER DECLARATION
//*********************************************************************************************************************

/// Timers to be used for timebase purposes or to fire IRQs.
/// They don't have associated CC channels.
pub(crate) struct BasicTimer<const ADR: usize> {
    cr1: RW<ADR, 0x00>,
    cr2: RW<ADR, 0x04>,
    dier: RW<ADR, 0xC0>,
    sr: RW<ADR, 0x10>,
    egr: RW<ADR, 0x14>,
    cnt: RW<ADR, 0x24>,
    psc: RW<ADR, 0x28>,
    arr: RW<ADR, 0x2C>,
}

make_timebase!(TIM6: BasicTimer, TIM6_ADR, u16, CountDir::Up);
make_timebase!(TIM7: BasicTimer, TIM7_ADR, u16, CountDir::Up);

impl ClockEnable for TIM6 {
    const CLK_EN_BIT: usize = 4;
    const CLK_EN_BUS: RccBus = RccBus::APB1_1;
}

impl ClockEnable for TIM7 {
    const CLK_EN_BIT: usize = 5;
    const CLK_EN_BUS: RccBus = RccBus::APB1_1;
}

/// TIM2/5 are 32bits, meanwhile TIM3/4 only 16bits.
pub(crate) struct GPTimerType1<const ADR: usize> {
    cr1: RW<ADR, 0x000>,
    cr2: RW<ADR, 0x004>,
    smcr: RW<ADR, 0x008>, 
    dier: RW<ADR, 0x0C0>,
    sr: RW<ADR, 0x010>,
    egr: RW<ADR, 0x014>,
    oc_ccmr1: RW<ADR, 0x018>,
    ic_ccmr1: RW<ADR, 0x018>,
    oc_ccmr2: RW<ADR, 0x01C>,
    ic_ccmr2: RW<ADR, 0x01C>,
    ccer: RW<ADR, 0x020>,
    cnt: RW<ADR, 0x024>,
    psc: RW<ADR, 0x028>,
    arr: RW<ADR, 0x02C>,
    ccr1: RW<ADR, 0x034>,
    ccr2: RW<ADR, 0x038>,
    ccr3: RW<ADR, 0x03C>,
    ccr4: RW<ADR, 0x040>,
    ecr: RW<ADR, 0x058>,
    tisel: RW<ADR, 0x05C>,
    af1: RW<ADR, 0x060>,
    af2: RW<ADR, 0x064>,
    dcr: RW<ADR, 0x3DC>,
    dmar: RW<ADR, 0x3E0>,
}

make_timebase!(TIM2: GPTimerType1, TIM2_ADR, u32, CountDir::UpDown);
make_capturecompare!(TIM2 => CC: 1);
make_capturecompare!(TIM2 => CC: 2);
make_capturecompare!(TIM2 => CC: 3);
make_capturecompare!(TIM2 => CC: 4);

make_timebase!(TIM3: GPTimerType1, TIM3_ADR, u16, CountDir::UpDown);
make_capturecompare!(TIM3 => CC: 1);
make_capturecompare!(TIM3 => CC: 2);
make_capturecompare!(TIM3 => CC: 3);
make_capturecompare!(TIM3 => CC: 4);

make_timebase!(TIM4: GPTimerType1, TIM4_ADR, u16, CountDir::UpDown);
make_capturecompare!(TIM4 => CC: 1);
make_capturecompare!(TIM4 => CC: 2);
make_capturecompare!(TIM4 => CC: 3);
make_capturecompare!(TIM4 => CC: 4);


make_timebase!(TIM5: GPTimerType1, TIM5_ADR, u32, CountDir::UpDown);
make_capturecompare!(TIM5 => CC: 1);
make_capturecompare!(TIM5 => CC: 2);
make_capturecompare!(TIM5 => CC: 3);
make_capturecompare!(TIM5 => CC: 4);


impl ClockEnable for TIM2 {
    const CLK_EN_BIT: usize = 0;
    const CLK_EN_BUS: RccBus = RccBus::APB1_1;
}

impl ClockEnable for TIM3 {
    const CLK_EN_BIT: usize = 1;
    const CLK_EN_BUS: RccBus = RccBus::APB1_1;
}

impl ClockEnable for TIM4 {
    const CLK_EN_BIT: usize = 2;
    const CLK_EN_BUS: RccBus = RccBus::APB1_1;
}

impl ClockEnable for TIM5 {
    const CLK_EN_BIT: usize = 3;
    const CLK_EN_BUS: RccBus = RccBus::APB1_1;
}


/// Timers with good amount of functionalities, but few CC channels.
pub(crate) struct GPTimerType2<const ADR: usize> {
    cr1: RW<ADR, 0x000>,
    cr2: RW<ADR, 0x004>,
    smcr: RW<ADR, 0x008>, 
    dier: RW<ADR, 0x0C0>,
    sr: RW<ADR, 0x010>,
    egr: RW<ADR, 0x014>,
    oc_ccmr1: RW<ADR, 0x018>,
    ic_ccmr1: RW<ADR, 0x018>,
    oc_ccmr2: RW<ADR, 0x01C>,
    ic_ccmr2: RW<ADR, 0x01C>,
    ccer: RW<ADR, 0x020>,
    cnt: RW<ADR, 0x024>,
    psc: RW<ADR, 0x028>,
    arr: RW<ADR, 0x02C>,
    rcr: RW<ADR, 0x030>,
    ccr1: RW<ADR, 0x034>,
    ccr2: RW<ADR, 0x038>,
    bdtr: RW<ADR, 0x044>,
    dtr2: RW<ADR, 0x054>,
    tisel: RW<ADR, 0x05C>,
    af1: RW<ADR, 0x060>,
    af2: RW<ADR, 0x064>,
    dcr: RW<ADR, 0x3DC>,
    dmar: RW<ADR, 0x3E0>,
}

make_timebase!(TIM15: GPTimerType2, TIM15_ADR, u16, CountDir::Up);
make_capturecompare!(TIM15 => CC: 1);
make_capturecompare!(TIM15 => CC: 2);

impl ClockEnable for TIM15 {
    const CLK_EN_BIT: usize = 16;
    const CLK_EN_BUS: RccBus = RccBus::APB2;
}


/// Timers with good amount of functionalities, but few CC channels.
pub(crate) struct GPTimerType3<const ADR: usize> {
    cr1: RW<ADR, 0x000>,
    cr2: RW<ADR, 0x004>,
    dier: RW<ADR, 0x0C0>,
    sr: RW<ADR, 0x010>,
    egr: RW<ADR, 0x014>,
    oc_ccmr1: RW<ADR, 0x018>,
    ic_ccmr1: RW<ADR, 0x018>,
    ccer: RW<ADR, 0x020>,
    cnt: RW<ADR, 0x024>,
    psc: RW<ADR, 0x028>,
    arr: RW<ADR, 0x02C>,
    rcr: RW<ADR, 0x030>,
    ccr1: RW<ADR, 0x034>,
    bdtr: RW<ADR, 0x044>,
    dtr2: RW<ADR, 0x054>,
    tisel: RW<ADR, 0x05C>,
    af1: RW<ADR, 0x060>,
    af2: RW<ADR, 0x064>,
    or1: RW<ADR, 0x068>,
    dcr: RW<ADR, 0x3DC>,
    dmar: RW<ADR, 0x3E0>,
}

make_timebase!(TIM16: GPTimerType3, TIM16_ADR, u16, CountDir::Up);
make_capturecompare!(TIM16 => CC: 1);

make_timebase!(TIM17: GPTimerType3, TIM17_ADR, u16, CountDir::Up);
make_capturecompare!(TIM17 => CC: 1);

impl ClockEnable for TIM16 {
    const CLK_EN_BIT: usize = 17;
    const CLK_EN_BUS: RccBus = RccBus::APB2;
}

impl ClockEnable for TIM17 {
    const CLK_EN_BIT: usize = 18;
    const CLK_EN_BUS: RccBus = RccBus::APB2;
}


/// Most advanced and feature-rich timers. They have 4 CC channels with IOs and 2 internal OC channels.
pub(crate) struct AdvancedTimer<const ADR: usize> {
    cr1: RW<ADR, 0x000>,
    cr2: RW<ADR, 0x004>,
    smcr: RW<ADR, 0x008>, 
    dier: RW<ADR, 0x0C0>,
    sr: RW<ADR, 0x010>,
    egr: RW<ADR, 0x014>,
    oc_ccmr1: RW<ADR, 0x018>,
    ic_ccmr1: RW<ADR, 0x018>,
    oc_ccmr2: RW<ADR, 0x01C>,
    ic_ccmr2: RW<ADR, 0x01C>,
    ccer: RW<ADR, 0x020>,
    cnt: RW<ADR, 0x024>,
    psc: RW<ADR, 0x028>,
    arr: RW<ADR, 0x02C>,
    rcr: RW<ADR, 0x030>,
    ccr1: RW<ADR, 0x034>,
    ccr2: RW<ADR, 0x038>,
    ccr3: RW<ADR, 0x03C>,
    ccr4: RW<ADR, 0x040>,
    bdtr: RW<ADR, 0x044>,
    ccr5: RW<ADR, 0x048>,
    ccr6: RW<ADR, 0x04C>,
    oc_ccmr3: RW<ADR, 0x050>,
    dtr2: RW<ADR, 0x054>,
    ecr: RW<ADR, 0x058>,
    tisel: RW<ADR, 0x05C>,
    af1: RW<ADR, 0x060>,
    af2: RW<ADR, 0x064>,
    dcr: RW<ADR, 0x3DC>,
    dmar: RW<ADR, 0x3E0>,
}

make_timebase!(TIM1: AdvancedTimer, TIM1_ADR, u16, CountDir::UpDown);
make_capturecompare!(TIM1 => CC: 1);
make_capturecompare!(TIM1 => CC: 2);
make_capturecompare!(TIM1 => CC: 3);
make_capturecompare!(TIM1 => CC: 4);
make_capturecompare!(TIM1 => OC: 5);
make_capturecompare!(TIM1 => OC: 6);

make_timebase!(TIM8: AdvancedTimer, TIM8_ADR, u16, CountDir::UpDown);
make_capturecompare!(TIM8 => CC: 1);
make_capturecompare!(TIM8 => CC: 2);
make_capturecompare!(TIM8 => CC: 3);
make_capturecompare!(TIM8 => CC: 4);
make_capturecompare!(TIM8 => OC: 5);
make_capturecompare!(TIM8 => OC: 6);

make_timebase!(TIM20: AdvancedTimer, TIM20_ADR, u16, CountDir::UpDown);
make_capturecompare!(TIM20 => CC: 1);
make_capturecompare!(TIM20 => CC: 2);
make_capturecompare!(TIM20 => CC: 3);
make_capturecompare!(TIM20 => CC: 4);
make_capturecompare!(TIM20 => OC: 5);
make_capturecompare!(TIM20 => OC: 6);


impl ClockEnable for TIM1 {
    const CLK_EN_BIT: usize = 11;
    const CLK_EN_BUS: RccBus = RccBus::APB2;
}

impl ClockEnable for TIM8 {
    const CLK_EN_BIT: usize = 13;
    const CLK_EN_BUS: RccBus = RccBus::APB2;
}

impl ClockEnable for TIM20 {
    const CLK_EN_BIT: usize = 20;
    const CLK_EN_BUS: RccBus = RccBus::APB2;
}

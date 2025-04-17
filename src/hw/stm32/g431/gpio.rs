use crate::kernel::registers::*;
use crate::hal::gpio::*;

use super::rcc::{ClockEnable, RccBus};

impl PinSpeed {
    fn code(self) -> usize {
        match self {
            PinSpeed::Slow => 0b00,
            PinSpeed::Medium => 0b01,
            PinSpeed::Fast => 0b10,
            PinSpeed::VeryFast => 0b11,
        }
    }
}


const GPIOA_ADR: usize = 0x4800_0000;
const GPIOB_ADR: usize = 0x4800_0400;
const GPIOC_ADR: usize = 0x4800_0800;
const GPIOD_ADR: usize = 0x4800_0C00;
const GPIOE_ADR: usize = 0x4800_1000;
const GPIOF_ADR: usize = 0x4800_1400;
const GPIOG_ADR: usize = 0x4800_1800;

pub(crate) struct Gpio<const ADR: usize> {
    mode: RW<ADR, 0x00>,
    otype: RW<ADR, 0x04>,
    ospeed: RW<ADR, 0x08>,
    pupd: RW<ADR, 0x0C>,
    id: RO<ADR, 0x10>,
    od: RW<ADR, 0x14>,
    bsr: WO<ADR, 0x18>,
    lck: RW<ADR, 0x1C>,
    afl: RW<ADR, 0x20>,
    afh: RW<ADR, 0x24>,
    br: WO<ADR, 0x28>,
}

macro_rules! create_gpio {
    ($Port:ident: $clock:expr) => {
        impl ClockEnable for $Port {
            const CLK_EN_BIT: usize = $clock;
            const CLK_EN_BUS: RccBus = RccBus::AHB2;
        }

        impl GpioPort for $Port {
            #[inline]
            fn init_port(self) {
                $Port::activate_clock();
            }

            #[inline]
            fn set_high(self, n: usize) {
                $Port::regs().bsr.set_bit(n);
            }
        
            #[inline]
            fn is_set_high(self, n: usize) -> bool {
                $Port::regs().od.read_bit(n)
            }
        
            #[inline]
            fn is_high(self, n: usize) -> bool {
                $Port::regs().id.read_bit(n)
            }
        
            #[inline]
            fn set_low(self, n: usize) {
                $Port::regs().br.set_bit(n);
            }
        
            #[inline]
            fn is_set_low(self, n: usize) -> bool {
                $Port::regs().od.read_bit(n) == false
            }
        
            #[inline]
            fn is_low(self, n: usize) -> bool {
                $Port::regs().id.read_bit(n) == false
            }
        
            #[inline]
            fn set_input(self, n: usize) {
                $Port::regs().mode.clear(0b11 << (n + n));
            }
        
            #[inline]
            fn set_nopull(self, n: usize) {
                $Port::regs().pupd.clear(0b11 << (n + n));
            }
            
            #[inline]
            fn set_pullup(self, n: usize) {
                $Port::regs().pupd.clear(0b11 << (n + n));
                $Port::regs().pupd.set(0b01 << (n + n));
            }
        
            #[inline]
            fn set_pulldown(self, n: usize) {
                $Port::regs().pupd.clear(0b11 << (n + n));
                $Port::regs().pupd.set(0b10 << (n + n));
            }
        
            #[inline]
            fn set_out_pushpull(self, n: usize) {
                $Port::regs().mode.clear(0b11 << (n + n));
                $Port::regs().mode.set(0b01 << (n + n));
                $Port::regs().otype.clear_bit(n);
            }
        
            #[inline]
            fn set_out_opendrain(self, n: usize) {
                $Port::regs().mode.clear(0b11 << (n + n));
                $Port::regs().mode.set(0b01 << (n + n));
                $Port::regs().otype.set_bit(n);
            }
        
            #[inline]
            fn set_speed(self, speed: usize, n: usize) {
                $Port::regs().ospeed.clear(0b11 << (n + n));
                $Port::regs().ospeed.set(speed << (n + n));
            }
        
            #[inline]
            fn set_alternate(self, alternate: usize, n: usize) {
                $Port::regs().mode.clear(0b11 << (n + n));
                $Port::regs().mode.set(0b10 << (n + n));
                match n {
                    0..=7 => { 
                        $Port::regs().afl.clear(0b1111 << (n << 2));
                        $Port::regs().afl.set(alternate << (n << 2));
                    },
                    8..=15 => { 
                        $Port::regs().afh.clear(0b1111 << ((n - 8) << 2));
                        $Port::regs().afh.set(alternate << ((n - 8) << 2));
                    },
                    _ => (),
                }
            }
        }
    }
}

use crate::make_port;
use core::marker::PhantomData;
make_port!(A: Gpio, GPIOA_ADR => [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
make_port!(B: Gpio, GPIOB_ADR => [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
make_port!(C: Gpio, GPIOC_ADR => [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
make_port!(D: Gpio, GPIOD_ADR => [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
make_port!(E: Gpio, GPIOD_ADR => [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
make_port!(F: Gpio, GPIOD_ADR => [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
make_port!(G: Gpio, GPIOD_ADR => [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);

create_gpio!(PORTA: 0);
create_gpio!(PORTB: 1);
create_gpio!(PORTC: 2);
create_gpio!(PORTD: 3);
create_gpio!(PORTE: 4);
create_gpio!(PORTF: 5);
create_gpio!(PORTG: 6);
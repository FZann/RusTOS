use crate::kernel::registers::*;
use crate::hal::gpio::*;
use core::convert::Infallible;

const GPIO0_ADR: usize = 0x400A_0000;
const GPIO1_ADR: usize = 0x400A_2000;

pub struct Gpio<const ADR: usize> {
    fsub0: RW<ADR, 0x0400>,
    fsub1: RW<ADR, 0x0404>,
    fpub0: RW<ADR, 0x0444>,
    fpub1: RW<ADR, 0x0448>,
    pwren: WO<ADR, 0x0800>,
    rstctl: RW<ADR, 0x0804>,
    stat: RO<ADR, 0x0814>,
    clkovr: RW<ADR, 0x1010>,
    pdbgctl: RW<ADR, 0x1018>,
    iidx: RO<ADR, 0x1020>,
    imask: RW<ADR, 0x1028>,
    ris: RO<ADR, 0x1030>,
    mis: RO<ADR, 0x1038>,
    iset: WO<ADR, 0x1040>,
    iclr: WO<ADR, 0x1048>,
    // TODO: inserisci altri registri per gli eventi
    evt_mode: RO<ADR, 0x10E0>,
    desc: RO<ADR, 0x10FC>,
    dout3_0: WO<ADR, 0x1200>,
    dout7_4: WO<ADR, 0x1204>,
    dout11_7: WO<ADR, 0x1208>,
    dout15_12: WO<ADR, 0x120C>,
    dout19_16: WO<ADR, 0x1210>,
    dout23_20: WO<ADR, 0x1214>,
    dout27_24: WO<ADR, 0x1218>,
    dout31_28: WO<ADR, 0x121C>,
    dout31_0: RW<ADR, 0x1280>,
    doutset31_0: WO<ADR, 0x1290>,
    doutclr31_0: WO<ADR, 0x12A0>,
    douttgl31_0: WO<ADR, 0x12B0>,
    doe31_0: RW<ADR, 0x12C0>,
    doeset31_0: WO<ADR, 0x12D0>,
    doeclr31_0: WO<ADR, 0x12E0>,
    din3_0: RO<ADR, 0x1300>,
    din7_4: RO<ADR, 0x1304>,
    din11_7: RO<ADR, 0x1308>,
    din15_12: RO<ADR, 0x130C>,
    din19_16: RO<ADR, 0x1310>,
    din23_20: RO<ADR, 0x1314>,
    din27_24: RO<ADR, 0x1318>,
    din31_28: RO<ADR, 0x131C>,
    din31_0: RO<ADR, 0x1380>,
    polarity15_0: RW<ADR, 0x1390>,
    polarity31_16: RW<ADR, 0x13A0>,
    ctl: RW<ADR, 0x1400>,
    fastwake: RW<ADR, 0x1404>,
    sub0cfg: RW<ADR, 0x1500>,
    filteren15_0: RW<ADR, 0x1508>,
    filteren31_16: RW<ADR, 0x150C>,
    dmamask: RW<ADR, 0x1510>,
    sub1cfg: RW<ADR, 0x1520>,
}

impl<const ADR: usize> Gpio<ADR> {
    fn power_on(&mut self) {
        self.pwren.write(0x26000000);
        self.pwren.write(1);
    }

    fn set_high(&mut self, n: usize) {
        self.dout31_0.set_bit(n);
    }

    fn set_low(&mut self, n: usize) {
        self.dout31_0.clear_bit(n);
    }

    fn toggle(&mut self, n: usize) {
        self.douttgl31_0.set_bit(n);
    }

    fn is_high(&mut self, n: usize) -> bool {
        self.din31_0.read_bit(n) == true
    }

    fn is_set_high(&mut self, n: usize) -> bool {
        self.dout31_0.read_bit(n) == true
    }

    fn is_low(&mut self, n: usize) -> bool {
        self.din31_0.read_bit(n) == false
    }

    fn is_set_low(&mut self, n: usize) -> bool {
        self.dout31_0.read_bit(n) == false
    }

    fn set_input(&mut self, n: usize) {
        self.doe31_0.clear_bit(n);
    }

    fn set_out_pushpull(&mut self, n: usize) {
        let pincm0 = RW::<0x4042_8000, 0x04>::new();
        pincm0.set(1);
        self.doe31_0.set_bit(n);
    }
}



macro_rules! make_gpios {
    ($peripheral:ident: $regs:ident, $addr:expr, $port:expr, $clock:expr) => {
        pub struct $peripheral;

        impl crate::kernel::registers::Peripheral for $peripheral {
            type Registers = $regs<$addr>;
            const ADR: usize = $addr;
        }
        
        impl $peripheral {
            pub fn activate_clock() {
                $peripheral::regs().power_on();
            }
            
            pub const fn pin<const N: usize, M: Mode>() -> Pin<$port, N, M> {
                Pin::new()
            }
        }

        impl<const N: usize> Pin<$port, N, Output<PushPull>> {
            #[inline]
            pub fn init(&mut self) -> &mut Self {
                $peripheral::regs().set_out_pushpull(N);
                self
            }
        }

        impl<const N: usize> Pin<$port, N, Output<OpenDrain<NoPull>>> {
            #[inline]
            pub fn init(&mut self) -> &mut Self {
                //$peripheral::regs().set_out_opendrain(N);
                //$peripheral::regs().set_nopull(N);
                self
            }
        }

        impl<const N: usize> Pin<$port, N, Output<OpenDrain<PullUp>>> {
            #[inline]
            pub fn init(&mut self) -> &mut Self {
                //$peripheral::regs().set_out_opendrain(N);
                //$peripheral::regs().set_pullup(N);
                self
            }
        }

        impl<const N: usize> Pin<$port, N, Input<NoPull>> {
            #[inline]
            pub fn init(&mut self) -> &mut Self {
                //$peripheral::regs().set_input(N);
                //$peripheral::regs().set_nopull(N); 
                self
            }
        }

        impl<const N: usize> Pin<$port, N, Input<PullUp>> {
            #[inline]
            pub fn init(&mut self) -> &mut Self {
                //$peripheral::regs().set_input(N); 
                //$peripheral::regs().set_pullup(N); 
                self
            }
        }

        impl<const N: usize> Pin<$port, N, Input<PullDown>> {
            #[inline]
            pub fn init(&mut self) -> &mut Self {
                //$peripheral::regs().set_input(N); 
                //$peripheral::regs().set_pulldown(N); 
                self
            }
        }
        
        impl<const N: usize, T: Type> OutputPin for Pin<$port, N, Output<T>> {
            #[inline]
            fn set_low(&mut self) -> Result<(), Self::Error> {
                $peripheral::regs().set_low(N);
                Ok(())
            }
        
            #[inline]
            fn set_high(&mut self) -> Result<(), Self::Error> {
                $peripheral::regs().set_high(N);
                Ok(())
            }
        }

        impl<const N: usize, T: Type> StatefulOutputPin for Pin<$port, N, Output<T>> {
            #[inline]
            fn is_set_high(&mut self) -> Result<bool, Self::Error> {
                let res: bool;
                res = $peripheral::regs().is_set_high(N);
                Ok(res)
            }
        
            #[inline]
            fn is_set_low(&mut self) -> Result<bool, Self::Error> {
                let res: bool;
                res = $peripheral::regs().is_set_low(N);
                Ok(res)
            }

            #[inline]
            fn toggle(&mut self) -> Result<(), Infallible> {
                $peripheral::regs().toggle(N);
                Ok(())
            }
        }

        impl<const N: usize, P: Pull> InputPin for Pin<$port, N, Input<P>> {
            #[inline]
            fn is_low(&mut self) -> Result<bool, Self::Error> {
                let res: bool;
                res = $peripheral::regs().is_low(N);
                Ok(res)
            }
        
            #[inline]
            fn is_high(&mut self) -> Result<bool, Self::Error> {
                let res: bool;
                res = $peripheral::regs().is_high(N);
                Ok(res)
            }
        }
    };
}


make_gpios!(PORTA: Gpio, GPIO0_ADR, 'A', 1 << 0);
make_gpios!(PORTB: Gpio, GPIO1_ADR, 'B', 1 << 1);


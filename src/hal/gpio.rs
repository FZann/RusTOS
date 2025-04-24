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


use core::marker::PhantomData;
pub use embedded_hal::digital::*;
pub use crate::hw::gpio::*;
use crate::kernel::registers::Peripheral;

// *************** PIN TYPES ***************
pub trait Mode {}
pub trait Pull {}
pub trait Type {}

pub struct Uninit;
pub struct Input<P: Pull>(PhantomData<P>);
pub struct Output<T: Type>(PhantomData<T>);
pub struct Alternate<AF: AltFunc>(PhantomData<AF>);
pub struct PushPull;
pub struct OpenDrain<P: Pull>(PhantomData<P>);

impl Type for PushPull {}
impl<P: Pull> Type for OpenDrain<P> {}

// *************** PIN PULLS ***************

pub struct NoPull;
pub struct PullDown;
pub struct PullUp;

impl Pull for NoPull {}
impl Pull for PullDown {}
impl Pull for PullUp {}


// *************** MODE IMPLEMENTATION ***************

impl Mode for Uninit {}

impl<P: Pull> Mode for Input<P> {}

impl<T: Type> Mode for Output<T> {}

impl<AF: AltFunc> Mode for Alternate<AF> {}


// *************** PIN ALTERNATE FUNCTIONS ***************
pub trait AltFunc {
    const MASK: usize;
}

pub struct AF0;
pub struct AF1;
pub struct AF2;
pub struct AF3;
pub struct AF4;
pub struct AF5;
pub struct AF6;
pub struct AF7;
pub struct AF8;
pub struct AF9;
pub struct AF10;
pub struct AF11;
pub struct AF12;
pub struct AF13;
pub struct AF14;
pub struct AF15;

impl AltFunc for AF0 {
    const MASK: usize = 0;    
}
impl AltFunc for AF1 {
    const MASK: usize = 1;
}
impl AltFunc for AF2 {
    const MASK: usize = 2;
}
impl AltFunc for AF3 {
    const MASK: usize = 3;
}
impl AltFunc for AF4 {
    const MASK: usize = 4;
}
impl AltFunc for AF5 {
    const MASK: usize = 5;
}
impl AltFunc for AF6 {
    const MASK: usize = 6;
}
impl AltFunc for AF7 {
    const MASK: usize = 7;
}
impl AltFunc for AF8 {
    const MASK: usize = 8;
}
impl AltFunc for AF9 {
    const MASK: usize = 9;
}
impl AltFunc for AF10 {
    const MASK: usize = 10;
}
impl AltFunc for AF11 {
    const MASK: usize = 11;
}
impl AltFunc for AF12 {
    const MASK: usize = 12;
}
impl AltFunc for AF13 {
    const MASK: usize = 13;
}
impl AltFunc for AF14 {
    const MASK: usize = 14;
}
impl AltFunc for AF15 {
    const MASK: usize = 15;
}


// *************** OTHER PIN TRAITS ***************
#[derive(Debug, Clone, Copy)]
pub enum PinSpeed {
    Slow,
    Medium,
    Fast,
    VeryFast,
}

pub trait OutputSpeed {
    fn set_speed(&mut self, speed: PinSpeed);
}


// *************** PIN STRUCT AND EXPORTS ***************

pub(crate) trait GpioPort: Peripheral {
    fn init_port(self);

    fn set_high(self, n: usize);
    fn is_set_high(self, n: usize) -> bool;

    fn is_high(self, n: usize) -> bool;
    fn set_low(self, n: usize);

    fn is_set_low(self, n: usize) -> bool;
    fn is_low(self, n: usize) -> bool;

    fn set_input(self, n: usize);
    fn set_nopull(self, n: usize);
    fn set_pullup(self, n: usize);
    fn set_pulldown(self, n: usize);
    fn set_out_pushpull(self, n: usize);
    fn set_out_opendrain(self, n: usize);
    fn set_speed(self, speed: usize, n: usize);
    fn set_alternate(self, alternate: usize, n: usize);
}

pub trait Pin {
    /// GpioPort being private is voluntary
    /// User should not access gpio port directly, but should use Pins
    #[allow(private_bounds)]
    type Port: GpioPort;

    const PORT: Self::Port;
    const N: usize;

    fn port(&self) -> Self::Port {
        Self::PORT
    }
    
    fn num(&self) -> usize { 
        Self::N 
    }
} 

pub trait PinSetup: Pin {
    fn init(&mut self) -> &mut Self;
}

#[macro_export]
macro_rules! make_port {
    ($Port:ident: $Regs:ident, $addr:expr => [$($n:expr),+]) => {
        paste::paste! {
            pub struct [<PORT $Port>];
            
            impl [<PORT $Port>] {
                $(pub const [<P $Port $n>]: [<P $Port $n>]<Uninit> = [<P $Port $n>](PhantomData::<Uninit>);)+

                #[inline]
                pub fn init_port() {
                    [<PORT $Port>].init_port();
                }
            }
            
            impl crate::kernel::registers::Peripheral for [<PORT $Port>] {
                type Registers = $Regs<$addr>;
                const ADR: usize = $addr;
            }
        }
        
        paste::paste! {
            $(
                pub struct [<P $Port $n>]<M: Mode>(PhantomData<M>);

                impl<M: Mode> [<P $Port $n>]<M> {
                    pub const fn allocate() -> Self {
                        Self(PhantomData)
                    }

                    #[inline]
                    pub fn out_pushpull(self) -> [<P $Port $n>]<Output<PushPull>> {
                        self.port().set_out_pushpull(Self::N);
                        self.port().set_nopull(Self::N);
                        [<P $Port $n>]::allocate()
                    }

                    #[inline]
                    pub fn out_opendrain_np(self) -> [<P $Port $n>]<Output<OpenDrain<NoPull>>> {
                        self.port().set_out_opendrain(Self::N);
                        self.port().set_nopull(Self::N);
                        [<P $Port $n>]::allocate()
                    }

                    #[inline]
                    pub fn out_opendrain_pu(self) -> [<P $Port $n>]<Output<OpenDrain<PullUp>>> {
                        self.port().set_out_opendrain(Self::N);
                        self.port().set_pullup(Self::N);
                        [<P $Port $n>]::allocate()
                    }

                    #[inline]
                    fn in_np(self) -> [<P $Port $n>]<Input<NoPull>> {
                        self.port().set_input(Self::N);
                        self.port().set_nopull(Self::N);
                        [<P $Port $n>]::allocate()
                    }

                    #[inline]
                    fn in_pu(self) -> [<P $Port $n>]<Input<PullUp>> {
                        self.port().set_input(Self::N);
                        self.port().set_pullup(Self::N);
                        [<P $Port $n>]::allocate()
                    }

                    #[inline]
                    fn in_pd(self) -> [<P $Port $n>]<Input<PullDown>> {
                        self.port().set_input(Self::N);
                        self.port().set_pulldown(Self::N);
                        [<P $Port $n>]::allocate()
                    }

                    #[inline]
                    fn altfunc<AF: AltFunc>(self) -> [<P $Port $n>]<Alternate<AF>> {
                        self.port().set_alternate(AF::MASK, Self::N);
                        [<P $Port $n>]::allocate()
                    }
                }

                impl<M: Mode> Pin for [<P $Port $n>]<M> {
                    type Port = [<PORT $Port>];

                    const PORT: Self::Port = [<PORT $Port>] {};
                    const N: usize = $n;
                }

                impl PinSetup for [<P $Port $n>]<Output<PushPull>> {
                    #[inline]
                    fn init(&mut self) -> &mut Self {
                        self.port().init_port();
                        self.port().set_out_pushpull(Self::N);
                        self.port().set_nopull(Self::N);
                        self
                    }
                }
                
                impl PinSetup for [<P $Port $n>]<Output<OpenDrain<NoPull>>> {
                    #[inline]
                    fn init(&mut self) -> &mut Self {
                        self.port().init_port();
                        self.port().set_out_opendrain(Self::N);
                        self.port().set_nopull(Self::N);
                        self
                    }
                }
                
                impl PinSetup for [<P $Port $n>]<Output<OpenDrain<PullUp>>> {
                    #[inline]
                    fn init(&mut self) -> &mut Self {
                        self.port().init_port();
                        self.port().set_out_opendrain(Self::N);
                        self.port().set_pullup(Self::N);
                        self
                    }
                }
                
                impl PinSetup for [<P $Port $n>]<Input<PullUp>> {
                    #[inline]
                    fn init(&mut self) -> &mut Self {
                        self.port().init_port();
                        self.port().set_input(Self::N);
                        self.port().set_pullup(Self::N);
                        self
                    }
                }
                
                impl PinSetup for [<P $Port $n>]<Input<NoPull>> {
                    #[inline]
                    fn init(&mut self) -> &mut Self {
                        self.port().init_port();
                        self.port().set_input(Self::N);
                        self.port().set_nopull(Self::N);
                        self
                    }
                }
                
                impl PinSetup for [<P $Port $n>]<Input<PullDown>> {
                    #[inline]
                    fn init(&mut self) -> &mut Self {
                        self.port().init_port();
                        self.port().set_input(Self::N);
                        self.port().set_pulldown(Self::N);
                        self
                    }
                }
                
                impl<AF: AltFunc> PinSetup for [<P $Port $n>]<Alternate<AF>> {
                    #[inline]
                    fn init(&mut self) -> &mut Self {
                        self.port().init_port();
                        self.port().set_alternate(AF::MASK, Self::N);
                        self
                    }
                }

                impl<M: Mode> ErrorType for [<P $Port $n>]<M> {
                    type Error = core::convert::Infallible;
                }
                
                impl<T: Type> OutputSpeed for [<P $Port $n>]<Output<T>> {
                    #[inline]
                    fn set_speed(&mut self, speed: PinSpeed) {
                        self.port().set_speed(speed.code(), Self::N);
                    }
                }
                
                impl<T: Type> OutputPin for [<P $Port $n>]<Output<T>> {
                    fn set_low(&mut self) -> Result<(), Self::Error> {
                        self.port().set_low(Self::N);
                        Ok(())
                    }
                
                    fn set_high(&mut self) -> Result<(), Self::Error> {
                        self.port().set_high(Self::N);
                        Ok(())
                    }
                }

                impl<T: Type> StatefulOutputPin for [<P $Port $n>]<Output<T>> {
                    #[inline]
                    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
                        let res: bool;
                        res = self.port().is_set_high(Self::N);
                        Ok(res)
                    }

                    #[inline]
                    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
                        let res: bool;
                        res = self.port().is_set_low(Self::N);
                        Ok(res)
                    }
                }
                
                impl<P: Pull> InputPin for [<P $Port $n>]<Input<P>> {
                    #[inline]
                    fn is_low(&mut self) -> Result<bool, Self::Error> {
                        let res: bool;
                        res = self.port().is_low(Self::N);
                        Ok(res)
                    }
                
                    #[inline]
                    fn is_high(&mut self) -> Result<bool, Self::Error> {
                        let res: bool;
                        res = self.port().is_high(Self::N);
                        Ok(res)
                    }
                }
            )+
        }
    };
}

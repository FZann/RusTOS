use crate::kernel::CritSect;

pub mod gpio;
pub mod rcc;


pub trait Peripheral {
    type Registers;
    const PTR: *const Self::Registers;

    #[inline(always)]
    unsafe fn regs<'r>() -> &'r mut Self::Registers {
        core::mem::transmute(Self::PTR)
    }

    #[inline(always)]
    fn get(_cs: &CritSect) -> &mut Self::Registers {
        unsafe { Self::regs() }
    }

    /// Esecuzione di una funzione racchiusa in una critical section
    fn with(mut f: impl FnMut(&mut Self::Registers)) {
        let cs = CritSect::activate();
        f(unsafe { Self::regs() });
        drop(cs);
    }
}


#[macro_export] macro_rules! make_peripheral {
    ($peripheral: ident: $addr:expr) => {
        impl crate::peripherals::Peripheral for $peripheral {
            type Registers = Self;
            const PTR: *const Self::Registers = $addr as *const Self::Registers;
        }
    };
}

#[macro_export] macro_rules! make_peripherals {
    ($peripheral: ident: $addr:expr, $regs: ident) => {
        pub struct $peripheral;

        impl crate::peripherals::Peripheral for $peripheral {
            type Registers = $regs;
            const PTR: *const Self::Registers = $addr as *const Self::Registers;
        }
    };
}
use crate::kernel::CriticalSection;
use core::cell::UnsafeCell;
use core::ptr;

#[repr(transparent)]
pub struct Reg<T: Copy> {
    reg: UnsafeCell<T>,
}

impl<T: Copy> Reg<T> {
    /// Creates a new `Reg` containing the given value
    pub const fn new(value: T) -> Self {
        Reg { reg: UnsafeCell::new(value) }
    }

    /// Returns a copy of the contained value
    #[inline(always)]
    pub fn get(&self) -> T {
        unsafe { ptr::read_volatile(self.reg.get()) }
    }

    /// Sets the contained value
    #[inline(always)]
    pub fn set(&self, value: T) {
        unsafe { ptr::write_volatile(self.reg.get(), value) }
    }

    /// Returns a raw pointer to the underlying data in the cell
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut T {
        self.reg.get().cast()
    }
}

/// Read-Only register
pub struct RO<T: Copy> {
    register: Reg<T>,
}

impl<T: Copy> RO<T> {
    /// Reads the value of the register
    #[inline(always)]
    pub fn read(&self) -> T {
        self.register.get()
    }
}

/// Read-Write register
pub struct RW<T: Copy> {
    register: Reg<T>,
}

impl<T: Copy> RW<T> {
    /// Performs a read-modify-write operation
    ///
    /// NOTE: `unsafe` because writes to a register are side effectful
    #[inline(always)]
    pub unsafe fn modify<F>(&self, f: F)
        where F: FnOnce(T) -> T
    {
        self.register.set(f(self.register.get()));
    }

    /// Reads the value of the register
    #[inline(always)]
    pub fn read(&self) -> T {
        self.register.get()
    }

    /// Writes a `value` into the register
    ///
    /// NOTE: `unsafe` because writes to a register are side effectful
    #[inline(always)]
    pub unsafe fn write(&self, value: T) {
        self.register.set(value)
    }
}

/// Write-Only register
pub struct WO<T: Copy> {
    register: Reg<T>,
}

impl<T: Copy> WO<T> {
    /// Writes `value` into the register
    ///
    /// NOTE: `unsafe` because writes to a register are side effectful
    #[inline(always)]
    pub unsafe fn write(&self, value: T) {
        self.register.set(value)
    }
}


pub trait Peripheral {
    type Registers;
    const PTR: *const Self::Registers;

    #[inline(always)]
    unsafe fn regs<'r>() -> &'r mut Self::Registers {
        core::mem::transmute(Self::PTR)
    }

    #[inline(always)]
    fn get(_cs: &CriticalSection) -> &mut Self::Registers {
        unsafe { Self::regs() }
    }

    /// Esecuzione di una funzione racchiusa in una critical section
    fn with(mut f: impl FnMut(&mut Self::Registers)) {
        let cs = CriticalSection::activate();
        f(unsafe { Self::regs() });
        drop(cs);
    }
}


#[macro_export] macro_rules! make_peripheral {
    ($peripheral: ident: $addr:expr, $regs: ident) => {
        pub struct $peripheral;

        impl crate::kernel::registers::Peripheral for $peripheral {
            type Registers = $regs;
            const PTR: *const Self::Registers = $addr as *const Self::Registers;
        }
    };
}
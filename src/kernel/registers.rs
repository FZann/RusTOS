use crate::kernel::CriticalSection;
use core::cell::UnsafeCell;
use core::ptr::NonNull;
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
    const MEM: NonNull<Self::Registers>;

    #[inline(always)]
    unsafe fn regs(&self) -> &mut Self::Registers {
        &mut *Self::MEM.as_ptr()
    }

    #[inline(always)]
    fn get_access<'cs>(&'cs self, _cs: &'cs CriticalSection) -> &'cs mut Self::Registers {
        unsafe { self.regs() }
    }

    /// Esecuzione di una funzione, racchiusa in una critical section
    fn with(&self, mut f: impl FnMut(&mut Self::Registers)) {
        let cs = CriticalSection::activate();
        f(unsafe { self.regs() });
        drop(cs);
    }
}


#[macro_export] macro_rules! make_peripheral {
    ($peripheral: ident: $addr:expr, $regs: ident => $static: ident) => {
        pub mod $peripheral {
            use core::ptr::NonNull;
            use crate::kernel::registers::Peripheral;
            use super::$regs;

            pub struct $peripheral<const ADDR: usize>;
            pub static mut $static: $peripheral<$addr> = $peripheral::define();

            impl<const ADDR: usize> $peripheral<ADDR> {
                pub const fn define() -> Self {
                    $peripheral::<ADDR>
                }
            }

            impl<const ADDR: usize> Peripheral for $peripheral<ADDR> {
                type Registers = $regs;
                const MEM: NonNull<Self::Registers> = NonNull::new(ADDR as *mut Self::Registers).unwrap();
            }
        }
    };
}
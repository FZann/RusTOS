pub mod gpio;

use core::mem::transmute;


pub(crate) trait MemMappedRegister {
    type Register;
    const ADDRESS: *mut Self::Register;
    fn as_mut_ref() -> &'static mut Self::Register {
        unsafe { transmute(Self::ADDRESS) }
    }
}

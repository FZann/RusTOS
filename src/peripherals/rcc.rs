use volatile_register::RW;


#[repr(C)]
pub struct RCC {
    cr: RW<u32>,
    cfgr: RW<u32>,
    
}

crate::make_peripheral!(RCC: 0x4002_1000);
use crate::kernel::registers::*;

const IOMUX_ADR: usize = 0x4042_8000;

trait IOValues {
    #[allow(non_upper_case_globals)]
    const PINCMx: usize;
    const PF_ID: usize;
}

struct IOMUX {
    pincm: RWArea<IOMUX_ADR, 0x04, 60>,
}


impl Peripheral for IOMUX {
    type Registers = IOMUX;
    const ADR: usize = IOMUX_ADR;
}


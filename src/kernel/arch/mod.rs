#[cfg(cortex_m)]
pub(crate) mod arm;
#[cfg(cortex_m)]
pub(crate) use arm as core;

#[cfg(riscv)]
pub(crate) mod riscv;
#[cfg(riscv)]
pub(crate) use riscv as core;

#[cfg(mips)]
pub(crate) mod mips;
#[cfg(mips)]
pub(crate) use mips as core;


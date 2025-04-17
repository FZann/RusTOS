#[cfg(feature = "stm32")]
mod stm32;
#[cfg(feature = "g431")]
pub(crate) use stm32::g431::*;


#[cfg(feature = "mspm0")]
mod ti_mspm0;
#[cfg(feature = "m0g3507")]
pub(crate) use ti_mspm0::g3507::*;

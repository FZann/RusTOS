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


#[cfg(feature = "stm32")]
mod stm32;
#[cfg(feature = "g431")]
pub(crate) use stm32::g431::*;


#[cfg(feature = "mspm0")]
mod ti_mspm0;
#[cfg(feature = "m0g3507")]
pub(crate) use ti_mspm0::g3507::*;

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


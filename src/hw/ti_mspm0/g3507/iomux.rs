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


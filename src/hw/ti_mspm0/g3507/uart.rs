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

const UART0_ADR: usize = 0x4010_8000;
const UART1_ADR: usize = 0x4010_0000;
const UART2_ADR: usize = 0x4010_2000;
const UART3_ADR: usize = 0x4050_0000;

pub struct Uart<const ADR: usize> {

}


pub struct UART0;
impl Peripheral for UART0 {
    type Registers = Uart<UART0_ADR>;
    const ADR: usize = UART0_ADR;
}

pub struct UART1;
impl Peripheral for UART1 {
    type Registers = Uart<UART1_ADR>;
    const ADR: usize = UART1_ADR;
}

pub struct UART2;
impl Peripheral for UART2 {
    type Registers = Uart<UART2_ADR>;
    const ADR: usize = UART2_ADR;
}
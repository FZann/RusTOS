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
//! 
//! 
//! -------------------------------------------------------------------------------------------------------------------
//! --------------------------------- Hardware Abstraction Layer for RusTOS -------------------------------------------
//! -------------------------------------------------------------------------------------------------------------------
//! 
//! This crate contains code that helps to give a common API to access various peripherals
//! of various MCUs, exposing a unified architecture to build upon other things (eg: drivers).
//! 
//! If possible, we want to keep special functionalities of MCUs families: that could be achieved
//! with conditional compilation of some traits. Even though this could cost some portability,
//! using your MCU at full potential is still an objective of RusTOS: when is possible to use
//! hardware, just do it!
//! 

//*********************************************************************************************************************
// HARDWARE HAL
//*********************************************************************************************************************
pub mod gpio;
pub mod uart;
pub mod tim;
pub mod dma;


//*********************************************************************************************************************
// SOFTWARE HAL
//*********************************************************************************************************************

// Here we offer some software constructs that interacts with hardware: DMARingBuffer is one of these objects.
// 
// DMARingBuffer is a simple Ring Buffer (surprise!) but is served by a DMA channel. This construct is quite useful
// for receiving UARTs characters without CPU intervention, but it needs an UART IRQ that signals transmission end.
// This function is implemented by STM with UART timers and by NXP with UART "idle character count" IRQ.
// With these, you can receive an arbitraty amount of data without using CPU at all.
// DMARingBuffer is used with eg: MODBUS protocol to receive frames without using any CPU cycle.

pub mod membufs;
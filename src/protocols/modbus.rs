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



/// Useful link to aid development: https://unserver.xyz/modbus-guide/
/// [Modbus over Serial Line Specification and Implementation Guide V1.02](http://modbus.org/docs/Modbus_over_serial_line_V1_02.pdf), page 13
/// "The maximum size of a Modbus RTU frame is 256 bytes."

type SlaveID = u8;
type Address = u16;
type Quantity = u16;
type Value = u16;

#[repr(C)]
enum FunctionCode<'payload> {
    ReadCoils(Address, Quantity),
    ReadDiscreteInputs(Address, Quantity),
    ReadHoldingRegisters(Address, Quantity),
    ReadInputRegisters(Address, Quantity),
    WriteSingleCoil(Address, Value),
    WriteSingleRegister(Address, Value),
    WriteMultipleCoils(Address, Quantity, &'payload [u8]),
    WriteMultipleRegisters(Address, Quantity, &'payload [u8]),
}

impl<'a> FunctionCode<'a> {
    fn code(&self) -> u8 {
        match *self {
            FunctionCode::ReadCoils(_, _) => 0x01,
            FunctionCode::ReadDiscreteInputs(_, _) => 0x02,
            FunctionCode::ReadHoldingRegisters(_, _) => 0x03,
            FunctionCode::ReadInputRegisters(_, _) => 0x04,
            FunctionCode::WriteSingleCoil(_, _) => 0x05,
            FunctionCode::WriteSingleRegister(_, _) => 0x06,
            FunctionCode::WriteMultipleCoils(_, _, _) => 0x0f,
            FunctionCode::WriteMultipleRegisters(_, _, _) => 0x10,
        }
        // ReadExceptionStatus     = 0x07,
        // ReportSlaveId           = 0x11,
        // MaskWriteRegister       = 0x16,
        // WriteAndReadRegisters   = 0x17
    }
}

#[derive(Clone, Copy)]
enum Exception {
	// MODBUS compliant
	IllegalFunc = 1,
	IllegalAddr = 2,
	InvalidDataValue = 3,
	DevFailure = 4,
	ACK = 5,
	Busy = 6,
	NAK = 7,
	MemParityError = 8,
	GatewayPathUnavailable = 10,
	GatewayTargetFail = 11,

	// Personali - uso interno della libreria
	NoException = 0,
	InvalidFrame = 100,
	RxTimeout = 110,
	SwitchMode = 120,
	FrameSendBack = 121,
}

enum StackMode {
	Master,
	Slave,
	Switch,
}

/// Single bit status values, used in read or write coil functions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Coil {
    On,
    Off,
}

impl Coil {
    fn code(self) -> u16 {
        match self {
            Coil::On => 0xff00,
            Coil::Off => 0x0000,
        }
    }
}


impl From<bool> for Coil {
    fn from(b: bool) -> Coil {
        if b {
            Coil::On
        } else {
            Coil::Off
        }
    }
}

impl core::ops::Not for Coil {
    type Output = Coil;

    fn not(self) -> Coil {
        match self {
            Coil::On => Coil::Off,
            Coil::Off => Coil::On,
        }
    }
}

struct Command<'p> {
	id: SlaveID,
	fc: FunctionCode<'p>,
}

struct Frame {
    id: SlaveID,
    fc: u8,
}
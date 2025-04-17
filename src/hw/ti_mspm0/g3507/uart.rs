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
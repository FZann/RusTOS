pub mod iomux;
pub mod gpio;
pub mod uart;
pub mod dma;
pub mod tim;
pub mod spi;
pub mod i2c;
pub mod can;
pub mod adc;
pub mod dac;
pub mod comp;
pub mod opa;

use crate::kernel::{MHz, Vector};

pub(crate) const CPU_FREQUENCY: MHz = MHz::new(180);

#[no_mangle]
#[cfg(armv6m)]
#[link_section = ".vector_table_interrupts"]
static __INTERRUPTS: [Vector; 32] = [
    Vector { handler: INT_GROUP0_ISR },
    Vector { handler: INT_GROUP1_ISR },
    Vector { handler: TIMG8_ISR },
    Vector { handler: UART3_ISR },
    Vector { handler: ADC0_ISR },
    Vector { handler: ADC1_ISR },
    Vector { handler: CANFD0_ISR },
    Vector { handler: DAC0_ISR },
    Vector { reserved: 0 },
    Vector { handler: SPI0_ISR },
    Vector { handler: SPI1_ISR },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { handler: UART1_ISR },
    Vector { handler: UART2_ISR },
    Vector { handler: UART0_ISR },
    Vector { handler: TIMG0_ISR },
    Vector { handler: TIMG6_ISR },
    Vector { handler: TIMA0_ISR },
    Vector { handler: TIMA1_ISR },
    Vector { handler: TIMG7_ISR },
    Vector { handler: TIMG12_ISR },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { handler: I2C0_ISR },
    Vector { handler: I2C1_ISR },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { handler: AES_ISR },
    Vector { reserved: 0 },
    Vector { handler: RTC_ISR },
    Vector { handler: DMA_ISR },
];

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn INT_GROUP0_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn INT_GROUP1_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIMG8_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn UART3_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn ADC0_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn ADC1_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn CANFD0_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DAC0_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn SPI0_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn SPI1_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn UART1_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn UART2_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn UART0_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIMG0_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIMG6_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIMA0_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIMA1_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIMG7_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIMG12_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn I2C0_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn I2C1_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn AES_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn RTC_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DMA_ISR() {}




pub mod rcc;
pub mod gpio;
pub mod uart;
pub mod dma;
pub mod tim;
pub mod spi;
pub mod i2c;
pub mod can;
pub mod adc;
pub mod dac;
pub mod qspi;

use crate::kernel::{MHz, Vector};

pub(crate) const CPU_FREQUENCY: MHz = MHz::new(100);

#[no_mangle]
#[cfg(not(armv6m))]
#[link_section = ".vector_table_interrupts"]
static __INTERRUPTS: [Vector; 101] = [
    Vector { handler: WWDG_ISR },
    Vector { handler: PVD_PVM_ISR },
    Vector { handler: RTC_ISR },
    Vector { handler: RTC_WKUP_ISR },
    Vector { handler: FLASH_ISR },
    Vector { handler: RCC_ISR },
    Vector { handler: EXTI0_ISR },
    Vector { handler: EXTI1_ISR },
    Vector { handler: EXTI2_ISR },
    Vector { handler: EXTI3_ISR },
    Vector { handler: EXTI4_ISR },
    Vector { handler: DMA1_CH1_ISR },
    Vector { handler: DMA1_CH2_ISR },
    Vector { handler: DMA1_CH3_ISR },
    Vector { handler: DMA1_CH4_ISR },
    Vector { handler: DMA1_CH5_ISR },
    Vector { handler: DMA1_CH6_ISR },
    Vector { handler: DMA1_CH7_ISR },
    Vector { handler: ADC1_2_ISR },
    Vector { handler: USB_HP_ISR },
    Vector { handler: USB_LP_ISR },
    Vector { handler: FDCAN1_IT0_ISR },
    Vector { handler: FDCAN1_IT1_ISR },
    Vector { handler: EXTI9_5_ISR },
    Vector { handler: TIM1_BRK_TIM15_ISR },
    Vector { handler: TIM1_UPDT_TIM16_ISR },
    Vector { handler: TIM1_TRG_COM_ISR },
    Vector { handler: TIM1_CC_ISR },
    Vector { handler: TIM2_ISR },
    Vector { handler: TIM3_ISR },
    Vector { handler: TIM4_ISR },
    Vector { handler: I2C1_EV_EXTI23_ISR },
    Vector { handler: I2C1_ER_ISR },
    Vector { handler: I2C2_EV_EXTI24_ISR },
    Vector { handler: I2C2_ER_ISR },
    Vector { handler: SPI1_ISR },
    Vector { handler: SPI2_ISR },
    Vector { handler: USART1_EXTI25_ISR },
    Vector { handler: USART2_EXTI26_ISR },
    Vector { handler: USART3_EXTI28_ISR },
    Vector { handler: EXTI15_10_ISR },
    Vector { handler: RTC_ALARM_ISR },
    Vector { handler: USBWakeUP_EXTI18_ISR },
    Vector { handler: TIM8_BRK_ISR },
    Vector { handler: TIM8_UPDT_ISR },
    Vector { handler: TIM8_TRG_COM_ISR },
    Vector { handler: TIM8_CC_ISR },
    Vector { handler: ADC3_ISR },
    Vector { handler: FSMC_ISR },
    Vector { handler: LPTIM1_ISR },
    Vector { handler: TIM5_ISR },
    Vector { handler: SPI3_ISR },
    Vector { handler: UART4_ISR },
    Vector { handler: UART5_ISR },
    Vector { handler: TIM6_DACUNDER_ISR },
    Vector { handler: TIM7_DACUNDER_ISR },
    Vector { handler: DMA2_CH1_ISR },
    Vector { handler: DMA2_CH2_ISR },
    Vector { handler: DMA2_CH3_ISR },
    Vector { handler: DMA2_CH4_ISR },
    Vector { handler: DMA2_CH5_ISR },
    Vector { handler: ADC4_ISR },
    Vector { handler: ADC5_ISR },
    Vector { handler: UCPD1_ISR },
    Vector { handler: COMP1_2_3_EXTI_21_22_29_ISR },
    Vector { handler: COMP4_5_6_EXTI_30_31_32_ISR },
    Vector { handler: COMP7_EXTI33_ISR },
    Vector { handler: HRTIM_TIMA_ISR },
    Vector { handler: HRTIM_TIMB_ISR },
    Vector { handler: HRTIM_TIMC_ISR },
    Vector { handler: HRTIM_TIMD_ISR },
    Vector { handler: HRTIM_TIME_ISR },
    Vector { handler: HRTIM_TIM_FLT_ISR },
    Vector { handler: HRTIM_TIMF_ISR },
    Vector { handler: CRS_ISR },
    Vector { handler: SAI_ISR },
    Vector { handler: TIM20_BRK_ISR },
    Vector { handler: TIM20_UPDT_ISR },
    Vector { handler: TIM20_TRG_COM_ISR },
    Vector { handler: TIM20_CC_ISR },
    Vector { handler: FPU_ISR },
    Vector { handler: I2C4_EV_EXTI42_ISR },
    Vector { handler: I2C4_ER_ISR },
    Vector { handler: SPI4_ISR },
    Vector { handler: AES_ISR },
    Vector { handler: FDCAN2_INT0_ISR },
    Vector { handler: FDCAN2_INT1_ISR },
    Vector { handler: FDCAN3_INT0_ISR },
    Vector { handler: FDCAN3_INT1_ISR },
    Vector { handler: RNG_ISR },
    Vector { handler: LPUART_ISR },
    Vector { handler: I2C3_EV_EXTI27_ISR },
    Vector { handler: I2C3_ER_ISR },
    Vector { handler: DMAMUX_OVR_ISR },
    Vector { handler: QUADSPI_ISR },
    Vector { handler: DMA1_CH8_ISR },
    Vector { handler: DMA2_CH6_ISR },
    Vector { handler: DMA2_CH7_ISR },
    Vector { handler: DMA2_CH8_ISR },
    Vector { handler: CORDIC_ISR },
    Vector { handler: FMAC_ISR },
    ];


#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn WWDG_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn PVD_PVM_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn RTC_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn RTC_WKUP_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn FLASH_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn RCC_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn EXTI0_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn EXTI1_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn EXTI2_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn EXTI3_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn EXTI4_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DMA1_CH1_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DMA1_CH2_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DMA1_CH3_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DMA1_CH4_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DMA1_CH5_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DMA1_CH6_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DMA1_CH7_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn ADC1_2_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn USB_HP_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn USB_LP_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn FDCAN1_IT0_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn FDCAN1_IT1_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn EXTI9_5_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIM1_BRK_TIM15_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIM1_UPDT_TIM16_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIM1_TRG_COM_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIM1_CC_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIM2_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIM3_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIM4_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn I2C1_EV_EXTI23_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn I2C1_ER_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn I2C2_EV_EXTI24_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn I2C2_ER_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn SPI1_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn SPI2_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn USART1_EXTI25_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn USART2_EXTI26_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn USART3_EXTI28_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn EXTI15_10_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn RTC_ALARM_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn USBWakeUP_EXTI18_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIM8_BRK_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIM8_UPDT_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIM8_TRG_COM_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIM8_CC_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn ADC3_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn FSMC_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn LPTIM1_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIM5_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn SPI3_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn UART4_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn UART5_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIM6_DACUNDER_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIM7_DACUNDER_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DMA2_CH1_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DMA2_CH2_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DMA2_CH3_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DMA2_CH4_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DMA2_CH5_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn ADC4_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn ADC5_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn UCPD1_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn COMP1_2_3_EXTI_21_22_29_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn COMP4_5_6_EXTI_30_31_32_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn COMP7_EXTI33_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn HRTIM_TIMA_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn HRTIM_TIMB_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn HRTIM_TIMC_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn HRTIM_TIMD_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn HRTIM_TIME_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn HRTIM_TIM_FLT_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn HRTIM_TIMF_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn CRS_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn SAI_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIM20_BRK_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIM20_UPDT_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIM20_TRG_COM_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn TIM20_CC_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn FPU_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn I2C4_EV_EXTI42_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn I2C4_ER_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn SPI4_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn AES_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn FDCAN2_INT0_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn FDCAN2_INT1_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn FDCAN3_INT0_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn FDCAN3_INT1_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn RNG_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn LPUART_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn I2C3_EV_EXTI27_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn I2C3_ER_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DMAMUX_OVR_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn QUADSPI_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DMA1_CH8_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DMA2_CH6_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DMA2_CH7_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DMA2_CH8_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn CORDIC_ISR() {}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn FMAC_ISR() {}

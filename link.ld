/* Memoria del chip STM32F334R8T6, usato per lo sviluppo */
MEMORY
{
    /* NOTE 1 K = 1 KiBi = 1024 bytes */
    FLASH   (rx)    : ORIGIN = 0x08000000,   LENGTH = 64K
    CCMRAM  (xrw)   : ORIGIN = 0x10000000,   LENGTH = 4K
    RAM     (xrw)   : ORIGIN = 0x20000000,   LENGTH = 10K
    STACK   (xrw)   : ORIGIN = 0x20003000,   LENGTH = 2K
}

EXTERN(__ENTRY);
EXTERN(__ERROR_HANDLER);
EXTERN(__ARM_VECTORS);
EXTERN(__INTERRUPTS);

ENTRY(__ENTRY);

SECTIONS 
{
    __stack_start = ORIGIN(STACK) + SIZEOF(STACK);
    PROVIDE(ld_stack_start = __stack_start);

    .vector_table : AT(ORIGIN(FLASH)) {
        LONG(__stack_start);
        KEEP(*(.vector_table_arm_vectors));
        KEEP(*(.vector_table_interrupts));
        KEEP(*(.static_kernel_variables));
    } > FLASH 

    __vector_end = ADDR(.vector_table) + SIZEOF(.vector_table);

    .text : AT(__vector_end) {
        . = ALIGN(4);
        *(.text .text*)
        . = ALIGN(4);
    } > FLASH

    .rodata : {
        *(.rodata .rodata.*);
        . = ALIGN(4);
        __end_flash = .;
    } > FLASH

    /* usato per caricare i dati inizializzati */
    .data : ALIGN(4) {
        . = ALIGN(4);
        PROVIDE(ld_data_start = . );
        *(.data*)
        . = ALIGN(4);
        PROVIDE(ld_data_end = . );
    } > RAM AT > FLASH
    PROVIDE(ld_data = LOADADDR(.data));

    .bss (NOLOAD) : {
        . = ALIGN(4);
        PROVIDE(ld_bss_start = . );
        *(.bss*)
        PROVIDE(ld_bss_end = . );
        . = ALIGN(4);
    } > RAM

    /* ### .uninit */
    .uninit (NOLOAD) : ALIGN(4)
    {
        . = ALIGN(4);
        /* __suninit = .; */
        *(.uninit .uninit.*);
        . = ALIGN(4); 
        /* __euninit = .; */
    } > RAM


    /DISCARD/ :
    {
        /* Unused exception related info that only wastes space */
        *(.ARM.exidx);
        *(.ARM.exidx.*);
        *(.ARM.extab.*);
    }

}
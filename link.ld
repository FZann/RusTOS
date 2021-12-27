MEMORY
{
    /* NOTE 1 K = 1 KiBi = 1024 bytes */
    FLASH :   ORIGIN = 0x08000000, LENGTH = 256K
    RAM   :   ORIGIN = 0x20000000, LENGTH = 60K
    STACK :   ORIGIN = 0x2000F000, LENGTH = 4K
}

EXTERN(__ENTRY);
EXTERN(__ERROR_HANDLER);
EXTERN(__ARM_VECTORS);
EXTERN(__INTERRUPTS);

ENTRY(__ENTRY);

SECTIONS 
{
    __stack_start = ORIGIN(STACK) + SIZEOF(STACK);

    .vector_table : AT(ORIGIN(FLASH)) {
        LONG(__stack_start);
        KEEP(*(.vector_table_arm_vectors));
        KEEP(*(.vector_table_interrupts));
        KEEP(*(.static_kernel_variables));
    } > FLASH 

    __vector_end = ADDR(.vector_table) + SIZEOF(.vector_table);

    .text : AT(__vector_end) {
        . = ALIGN(4);
        *(.os_entry)
        *(.os_errorhandler)
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
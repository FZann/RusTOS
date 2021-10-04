MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  FLASH :   ORIGIN = 0x08000000, LENGTH = 256K
  RAM   :   ORIGIN = 0x20000000, LENGTH = 64K
}

EXTERN(__ENTRY);
EXTERN(__ERROR_HANDLER);
EXTERN(__ARM_VECTORS);
EXTERN(__INTERRUPTS);

ENTRY(__ENTRY);

SECTIONS 
{
    .vector_table : AT(ORIGIN(FLASH)) {
        LONG(__stack_start);
        *(.vector_table_arm_vectors)
        *(.vector_table_interrupts)
    } > FLASH 

    PROVIDE(__vector_end = ADDR(.vector_table) + SIZEOF(.vector_table));

    .text : AT(__vector_end) {
        . = ALIGN(4);
        *(.os_entry)
        *(.os_errorhandler)
        . = ALIGN(4);
        *(.text .text*)
        . = ALIGN(4);
    } > FLASH

    .rodata : {
        . = ALIGN(4);
        *(.rodata .rodata.*);
    } > FLASH

    /* ### .gnu.sgstubs
        This section contains the TrustZone-M veneers put there by the Arm GNU linker. */
    /* Security Attribution Unit blocks must be 32 bytes aligned. */
    /* Note that this pads the FLASH usage to 32 byte alignment. */
    .gnu.sgstubs : ALIGN(32)
    {
        . = ALIGN(32);
        __veneer_base = .;
        *(.gnu.sgstubs*)
        *(.glue_7t) *(.glue_7)
        *(.debug*)
        . = ALIGN(32);
        __veneer_limit = .;
    } > FLASH

    /* usato per caricare i dati inizializzati */
    ld_data = LOADADDR(.data);
    .data : {
        . = ALIGN(4);
        ld_data_start = . ;
        *(.data*)
        . = ALIGN(4);
        ld_data_end = . ;
    } > RAM AT > FLASH

    .bss (NOLOAD) : {
        . = ALIGN(4);
        ld_bss_start = . ;
        *(.bss*)
        ld_bss_end = . ;
        . = ALIGN(4);
    } > RAM

    __stack_start = ORIGIN(RAM) + SIZEOF(RAM) - 0x200;

    .stack : AT(__stack_start) {
        . = ALIGN(8);
    } > RAM

  /DISCARD/ :
  {
    /* Unused exception related info that only wastes space */
    *(.ARM.exidx);
    *(.ARM.exidx.*);
    *(.ARM.extab.*);
  }

}
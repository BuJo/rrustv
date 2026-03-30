OUTPUT_ARCH(riscv)
ENTRY(_start)

MEMORY {
    RAM : ORIGIN = 0x80000000, LENGTH = 128M
}

SECTIONS {
    . = 0x80000000;

    /* Entry stub goes first */
    .text : {
        KEEP(*(.text.entry))
        *(.text .text.*)
    } > RAM

    .rodata : { *(.rodata .rodata.*) } > RAM
    .data   : { *(.data   .data.*)   } > RAM

    .bss : {
        _bss_start = .;
        *(.bss .bss.* COMMON)
        _bss_end = .;
    } > RAM

    /* 64 KiB stack placed right after BSS */
    . = ALIGN(16);
    _stack_bottom = .;
    . += 0x10000;
    _stack_top = .;
}

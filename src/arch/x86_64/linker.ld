ENTRY(_start)

SECTIONS {
    . = 1M;

    .boot :
    {
        KEEP(*(.mbheader))
    }

    .text :
    {
        *(.text .text.*)
    }

    .rodata :
    {
        *(.rodata .rodata.*)
    }

    .data.rel.ro :
    {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
    }
}

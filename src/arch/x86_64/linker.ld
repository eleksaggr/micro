ENTRY(_start)

SECTIONS {
    . = 1M;

    .boot :
    {
        KEEP(*(.mbheader))
    }

    .text :
    {
        *(.text)
    }
}

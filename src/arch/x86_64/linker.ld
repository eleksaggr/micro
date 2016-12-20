ENTRY(_start)

SECTIONS {
    . = 1M;

    .boot :
    {
        *(.mbheader)
    }

    .text :
    {
        *(.text)
    }
}

ENTRY(_start)

SECTIONS
{
    . = 0x80000000;
    
    .text : {
        *(.text.init)
        *(.text)
    }
    
    .rodata : {
        *(.rodata)
    }
    
    .data : {
        *(.data)
    }
    
    .bss : {
        *(.bss)
    }
}

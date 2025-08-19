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

    _stack_top = ALIGN(0x1000);
    . = _stack_top + 0x8000;
    _stack_pointer = .;
    end = .;
    _end = .;
    _heap_start = ALIGN(0x1000);
}

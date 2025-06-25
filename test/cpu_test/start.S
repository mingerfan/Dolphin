.section .text.init
.global _start

_start:
    # Set up stack pointer
    li sp, 0x80000000

    # Call main
    call main

    # Exit program
    li a7, 93     # syscall number for exit
    li a0, 0      # exit code
    ecall

.section .bss
.align 4
stack_bottom:
.skip 4096
stack_top:

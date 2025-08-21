#include "uart.h"

void uart_init(void) {
    /* 当前不需要初始化控制寄存器，保留接口以便将来扩展 */
    (void)UART_STATUS_REG;
}

void uart_putc(char c) {
    *(volatile unsigned char *)UART_DATA_REG = (unsigned char)c;
}

void uart_puts(const char *s) {
    while (s && *s) {
        uart_putc(*s++);
    }
}

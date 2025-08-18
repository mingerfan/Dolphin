#include "dolphin_runtime.h"

// UART 寄存器地址 (来自配置文件)
#define UART_BASE 0x10000000
#define UART_DATA_REG (UART_BASE + 0x00)
#define UART_STATUS_REG (UART_BASE + 0x04)

static inline void uart_putc(char c) {
    volatile unsigned char *uart_data = (volatile unsigned char *)UART_DATA_REG;
    *uart_data = c;
}

static inline void uart_puts(const char *s) {
    while (*s) {
        uart_putc(*s++);
    }
}

int main() {
    uart_puts("Hello from MMIO UART!\n");
    uart_puts("MMIO 功能测试成功！\n");

    ctrap(0);
}

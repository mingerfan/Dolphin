#ifndef DOLPHIN_UART_H
#define DOLPHIN_UART_H

#include <device_config.h>

#ifndef UART_BASE
#define UART_BASE DEVICE_UART0_BASE
#endif

#define UART_DATA_REG  (UART_BASE + 0x00)
#define UART_STATUS_REG (UART_BASE + 0x04)

void uart_init(void);
void uart_putc(char c);
void uart_puts(const char *s);

#endif // DOLPHIN_UART_H

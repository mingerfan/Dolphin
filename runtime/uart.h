#ifndef DOLPHIN_UART_H
#define DOLPHIN_UART_H


#ifndef UART_BASE
#define UART_BASE 0x10000000UL
#endif

#define UART_DATA_REG  (UART_BASE + 0x00)
#define UART_STATUS_REG (UART_BASE + 0x04)

void uart_init(void);
void uart_putc(char c);
void uart_puts(const char *s);

#endif // DOLPHIN_UART_H

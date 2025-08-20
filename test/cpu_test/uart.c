#include "dolphin_runtime.h"
#include "uart.h"

int main() {
    uart_init();
    uart_puts("Hello from MMIO UART!\n");
    uart_puts("MMIO 功能测试成功！\n");

    return 0;
}

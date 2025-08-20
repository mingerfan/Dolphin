#include "timer.h"
#include "uart.h"
#include "klib.h"
#include <stdint.h>

/* 直接从 MMIO 地址读取 64 位时间（低位在低地址） */
uint64_t timer_get_us(void) {
    volatile uint64_t *ptr = (volatile uint64_t *)TIMER_CNT0_REG;
    return *ptr;
}


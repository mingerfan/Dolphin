#ifndef DOLPHIN_TIMER_H
#define DOLPHIN_TIMER_H

#include <stdint.h>

#ifndef TIMER_BASE
#define TIMER_BASE DEVICE_TIMER0_BASE
#endif

#define TIMER_CNT0_REG (TIMER_BASE + 0x00)

/* 返回设备提供的当前时间（单位：微秒） */
uint64_t timer_get_us(void);

#endif // DOLPHIN_TIMER_H

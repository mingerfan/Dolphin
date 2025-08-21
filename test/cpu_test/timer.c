#include "dolphin_runtime.h"
#include "timer.h"
#include <stdint.h>
#include <klib.h>

/*
 使用 runtime 提供的 timer_get_us / timer_print_sample 接口，
 打印中间结果并判断两次采样差值在阈值内视为通过。
*/

int main() {
    // 主动采样两次并比较差值
    uint64_t t1 = timer_get_us();

    // 简单忙等待
    volatile unsigned long i;
    for (i = 0; i < 50000UL; i++) {
        asm volatile("" ::: "memory");
    }

    uint64_t t2 = timer_get_us();
    uint64_t dt = (t2 >= t1) ? (t2 - t1) : 0;


    printf("dt: %u\n", (uint32_t)dt);

    // 判定阈值（例如 2 秒 = 2_000_000 us）视为通过
    const uint64_t THRESH = 2000000UL;
    if (dt > 0 && dt < THRESH) {
        return 0;
    } else {
        return 1;
    }

    // uint64_t t1 = timer_get_us();

    // while(timer_get_us() - t1 < 5000000);

}

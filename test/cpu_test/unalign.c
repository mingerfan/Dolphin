#include "trap.h"

volatile unsigned x = 0xffffffff;
volatile unsigned char buf[16];

int main() {

	for(int i = 0; i < 4; i++) {
	    // 由于手册和ref均不支持非对齐访问，我们在这里将buf + 3的访问改为buf + i * 4
		// 不知道是否正确:(
		*((volatile unsigned*)(buf + i * 4)) = 0xaabbccdd;

		x = *((volatile unsigned*)(buf + i * 4));
		check(x == 0xaabbccdd);

		buf[i * 4] = buf[i * 4 + 1] = buf[i * 4 + 2] = buf[i * 4 + 3] = 0;
	}

	return 0;
}

#include "trap.h"

char buf[128];

int main() {
	sprintf(buf, "%s", "Hello world!\n");
	printf("输出：%s", buf);
	check(strcmp(buf, "Hello world!\n") == 0);

	sprintf(buf, "%d + %d = %d\n", 1, 1, 2);
	printf("输出：%s", buf);
	check(strcmp(buf, "1 + 1 = 2\n") == 0);

	sprintf(buf, "%d + %d = %d\n", 2, 10, 12);
	printf("输出：%s", buf);
	check(strcmp(buf, "2 + 10 = 12\n") == 0);

	return 0;
}

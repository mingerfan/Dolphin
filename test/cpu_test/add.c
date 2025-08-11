// #include <stdio.h>

int main() {
    int a = 10;
    int b = 20;
    int sum = a + b;

    // printf("Adding numbers: %d + %d = %d\n", a, b, sum);
    asm volatile("mv a0, %0; ebreak" : :"r"(0));
}

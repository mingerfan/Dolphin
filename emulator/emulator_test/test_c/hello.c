int main() {
    int sum = 0;
    for (int i = 1; i <= 100; ++i) {
        sum += i;
    }
    asm volatile("mv a0, %0; ebreak" : :"r"(0));
}

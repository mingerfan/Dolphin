#include "../runtime/klib.h"

int main() {
    // Test string functions
    char str1[100] = "Hello ";
    char str2[] = "World!";

    printf("Testing klib functions:\n");

    // Test string functions
    printf("strlen: %u\n", (unsigned int)strlen(str1));
    strcat(str1, str2);
    printf("strcat result: %s\n", str1);

    char str3[100];
    strcpy(str3, "Copy test");
    printf("strcpy result: %s\n", str3);

    // Test comparison
    printf("strcmp result: %d\n", strcmp("abc", "abc"));
    printf("strcmp result: %d\n", strcmp("abc", "def"));

    // Test memory functions
    char buffer[20];
    memset(buffer, 'A', 10);
    buffer[10] = '\0';
    printf("memset result: %s\n", buffer);

    // Test stdlib functions
    printf("abs(-42): %d\n", abs(-42));
    printf("atoi(\"-12315\"): %d\n", atoi("-12315"));

    // Test malloc/free
    char *ptr = (char*)malloc(100);
    if (ptr) {
        strcpy(ptr, "malloc test");
        printf("malloc test: %s\n", ptr);
        free(ptr);
    }

    // Test random numbers
    srand(42);
    printf("Random numbers: %d %d %d\n", rand(), rand(), rand());

    // sprintf return test
    char buffer2[100];
    int res = sprintf(buffer2, "%d", 12345);
    printf("sprintf result: %s, ret: %d\n", buffer2, res);
    assert(res == 5);

    // Test sprintf with string
    res = sprintf(buffer2, "Hello %s!", "World");
    printf("sprintf string result: %s, ret: %d\n", buffer2, res);
    assert(res == 12);

    // Test sprintf with multiple format specifiers
    res = sprintf(buffer2, "Number: %d, Hex: %x", 42, 255);
    printf("sprintf multi result: %s, ret: %d\n", buffer2, res);
    assert(res == 19); // "Number: 42, Hex: ff" = 19 chars

    // Test long format specifiers (LP64 ABI)
    res = sprintf(buffer2, "Long: %ld", 1234567890L);
    printf("sprintf long result: %s, ret: %d\n", buffer2, res);
    assert(res == 16); // "Long: 1234567890" = 16 chars

    res = sprintf(buffer2, "ULong: %lu", 4294967295UL);
    printf("sprintf ulong result: %s, ret: %d\n", buffer2, res);
    assert(res == 17);

    res = sprintf(buffer2, "HexLong: %lx", 0xDEADBEEFL);
    printf("sprintf hexlong result: %s, ret: %d\n", buffer2, res);
    assert(res == 17);

    // Test snprintf with truncation
    char small_buf[10];
    res = snprintf(small_buf, sizeof(small_buf), "Very long string %d", 999);
    printf("snprintf truncated: %s, ret: %d\n", small_buf, res);
    assert(res == 20); // Should return the number of chars that would have been written
    assert(strlen(small_buf) == 9); // Only 9 chars fit (plus null terminator)

    // Test assert (should not trigger)
    assert(1 == 1);
    printf("Assert test passed\n");

    printf("All klib tests completed!\n");

    return 0;
}

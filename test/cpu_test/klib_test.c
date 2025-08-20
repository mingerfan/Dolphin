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
    
    // Test assert (should not trigger)
    assert(1 == 1);
    printf("Assert test passed\n");
    
    printf("All klib tests completed!\n");
    
    return 0;
}

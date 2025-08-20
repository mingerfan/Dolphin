#include "../runtime/klib.h"
#include <stdio.h>
#include <string.h>

int main() {
    char buffer[100];
    
    // Test sprintf
    sprintf(buffer, "Hello %s! Number: %d, Hex: %x", "World", 42, 255);
    printf("sprintf result: %s\n", buffer);
    
    // Test snprintf
    snprintf(buffer, sizeof(buffer), "Limited: %d %s", 123, "test");
    printf("snprintf result: %s\n", buffer);
    
    // Test with small buffer
    char small[10];
    snprintf(small, sizeof(small), "Very long string %d", 999);
    printf("snprintf small result: %s\n", small);
    
    return 0;
}

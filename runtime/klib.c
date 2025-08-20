#include "klib.h"
#include "uart.h"
#include "dolphin_runtime.h"

// Simple memory allocator state
static char heap[4096];
static size_t heap_pos = 0;

// Random number generator state
static unsigned int rand_seed = 1;

// ========== string.h ==========

void *memset(void *s, int c, size_t n) {
    unsigned char *p = (unsigned char *)s;
    while (n--) {
        *p++ = (unsigned char)c;
    }
    return s;
}

void *memcpy(void *dst, const void *src, size_t n) {
    unsigned char *d = (unsigned char *)dst;
    const unsigned char *s = (const unsigned char *)src;
    while (n--) {
        *d++ = *s++;
    }
    return dst;
}

void *memmove(void *dst, const void *src, size_t n) {
    unsigned char *d = (unsigned char *)dst;
    const unsigned char *s = (const unsigned char *)src;

    if (d < s) {
        // Copy forward
        while (n--) {
            *d++ = *s++;
        }
    } else {
        // Copy backward
        d += n;
        s += n;
        while (n--) {
            *--d = *--s;
        }
    }
    return dst;
}

int memcmp(const void *s1, const void *s2, size_t n) {
    const unsigned char *p1 = (const unsigned char *)s1;
    const unsigned char *p2 = (const unsigned char *)s2;

    while (n--) {
        if (*p1 != *p2) {
            return *p1 - *p2;
        }
        p1++;
        p2++;
    }
    return 0;
}

size_t strlen(const char *s) {
    size_t len = 0;
    while (*s++) {
        len++;
    }
    return len;
}

char *strcat(char *dst, const char *src) {
    char *d = dst;
    while (*d) d++; // Find end of dst
    while ((*d++ = *src++)); // Copy src
    return dst;
}

char *strcpy(char *dst, const char *src) {
    char *d = dst;
    while ((*d++ = *src++));
    return dst;
}

char *strncpy(char *dst, const char *src, size_t n) {
    char *d = dst;
    while (n && (*d++ = *src++)) {
        n--;
    }
    while (n--) {
        *d++ = '\0';
    }
    return dst;
}

int strcmp(const char *s1, const char *s2) {
    while (*s1 && (*s1 == *s2)) {
        s1++;
        s2++;
    }
    return *(unsigned char *)s1 - *(unsigned char *)s2;
}

int strncmp(const char *s1, const char *s2, size_t n) {
    while (n && *s1 && (*s1 == *s2)) {
        s1++;
        s2++;
        n--;
    }
    if (n == 0) {
        return 0;
    }
    return *(unsigned char *)s1 - *(unsigned char *)s2;
}

// ========== stdlib.h ==========

void srand(unsigned int seed) {
    rand_seed = seed;
}

int rand(void) {
    // Linear congruential generator
    rand_seed = rand_seed * 1103515245 + 12345;
    return (rand_seed / 65536) % 32768;
}

void *malloc(size_t size) {
    // Simple bump allocator
    if (heap_pos + size > sizeof(heap)) {
        return NULL; // Out of memory
    }
    void *ptr = heap + heap_pos;
    heap_pos += size;
    return ptr;
}

void free(void *ptr) {
    // Simple allocator doesn't support free
    (void)ptr;
}

int abs(int x) {
    return x < 0 ? -x : x;
}

int atoi(const char *nptr) {
    int result = 0;
    int sign = 1;

    // Skip whitespace
    while (*nptr == ' ' || *nptr == '\t' || *nptr == '\n' || *nptr == '\r') {
        nptr++;
    }

    // Handle sign
    if (*nptr == '-') {
        sign = -1;
        nptr++;
    } else if (*nptr == '+') {
        nptr++;
    }

    // Convert digits
    while (*nptr >= '0' && *nptr <= '9') {
        result = result * 10 + (*nptr - '0');
        nptr++;
    }

    return result * sign;
}

// ========== stdio.h ==========

// Helper function to write a character to UART
static void putchar_impl(char c) {
    uart_putc(c);
}

// Helper function to write a string to UART
static void puts_impl(const char *s) {
    while (*s) {
        putchar_impl(*s++);
    }
}

// Helper function to write an integer to UART
static void putint_impl(long n) {
    if (n < 0) {
        putchar_impl('-');
        n = -n;
    }

    if (n >= 10) {
        putint_impl(n / 10);
    }

    putchar_impl('0' + (n % 10));
}

// Helper function to write an unsigned integer to UART
static void putuint_impl(unsigned long n) {
    if (n >= 10) {
        putuint_impl(n / 10);
    }
    putchar_impl('0' + (n % 10));
}

// Helper function to write a hex number to UART
static void puthex_impl(unsigned int n) {
    if (n >= 16) {
        puthex_impl(n / 16);
    }
    char digit = n % 16;
    if (digit < 10) {
        putchar_impl('0' + digit);
    } else {
        putchar_impl('a' + digit - 10);
    }
}

// String formatting context for sprintf family functions
struct sprintf_ctx {
    char *str;
    size_t size;
    size_t pos;
};

// Helper function to write a character to string buffer
static void putchar_str(struct sprintf_ctx *ctx, char c) {
    if (ctx->pos < ctx->size - 1) {
        ctx->str[ctx->pos] = c;
    }
    ctx->pos++;
}

// Helper function to write a string to string buffer
static void puts_str(struct sprintf_ctx *ctx, const char *s) {
    while (*s) {
        putchar_str(ctx, *s++);
    }
}

// Helper function to write an integer to string buffer
static void putint_str(struct sprintf_ctx *ctx, long n) {
    if (n < 0) {
        putchar_str(ctx, '-');
        n = -n;
    }

    if (n >= 10) {
        putint_str(ctx, n / 10);
    }

    putchar_str(ctx, '0' + (n % 10));
}

// Helper function to write an unsigned integer to string buffer
static void putuint_str(struct sprintf_ctx *ctx, unsigned long n) {
    if (n >= 10) {
        putuint_str(ctx, n / 10);
    }
    putchar_str(ctx, '0' + (n % 10));
}

// Helper function to write a hex number to string buffer
static void puthex_str(struct sprintf_ctx *ctx, unsigned int n) {
    if (n >= 16) {
        puthex_str(ctx, n / 16);
    }
    char digit = n % 16;
    if (digit < 10) {
        putchar_str(ctx, '0' + digit);
    } else {
        putchar_str(ctx, 'a' + digit - 10);
    }
}

// Simple printf implementation (supports %d, %u, %x, %s, %c, %%)
int vprintf_impl(const char *format, va_list ap) {
    int count = 0;

    while (*format) {
        if (*format == '%') {
            format++;
            switch (*format) {
                case 'd': {
                    int n = va_arg(ap, int);
                    putint_impl(n);
                    count++;
                    break;
                }
                case 'u': {
                    unsigned int n = va_arg(ap, unsigned int);
                    putuint_impl(n);
                    count++;
                    break;
                }
                case 'x': {
                    unsigned int n = va_arg(ap, unsigned int);
                    puthex_impl(n);
                    count++;
                    break;
                }
                case 's': {
                    char *s = va_arg(ap, char *);
                    if (s) {
                        puts_impl(s);
                    }
                    count++;
                    break;
                }
                case 'c': {
                    int c = va_arg(ap, int);
                    putchar_impl((char)c);
                    count++;
                    break;
                }
                case '%': {
                    putchar_impl('%');
                    count++;
                    break;
                }
                default:
                    putchar_impl('%');
                    putchar_impl(*format);
                    count += 2;
                    break;
            }
        } else {
            putchar_impl(*format);
            count++;
        }
        format++;
    }

    return count;
}

// String formatting implementation (supports %d, %u, %x, %s, %c, %%)
static int vsprintf_impl(struct sprintf_ctx *ctx, const char *format, va_list ap) {
    int count = 0;

    while (*format) {
        if (*format == '%') {
            format++;
            switch (*format) {
                case 'd': {
                    int n = va_arg(ap, int);
                    putint_str(ctx, n);
                    count++;
                    break;
                }
                case 'u': {
                    unsigned int n = va_arg(ap, unsigned int);
                    putuint_str(ctx, n);
                    count++;
                    break;
                }
                case 'x': {
                    unsigned int n = va_arg(ap, unsigned int);
                    puthex_str(ctx, n);
                    count++;
                    break;
                }
                case 's': {
                    char *s = va_arg(ap, char *);
                    if (s) {
                        puts_str(ctx, s);
                    }
                    count++;
                    break;
                }
                case 'c': {
                    int c = va_arg(ap, int);
                    putchar_str(ctx, (char)c);
                    count++;
                    break;
                }
                case '%': {
                    putchar_str(ctx, '%');
                    count++;
                    break;
                }
                default:
                    putchar_str(ctx, '%');
                    putchar_str(ctx, *format);
                    count += 2;
                    break;
            }
        } else {
            putchar_str(ctx, *format);
            count++;
        }
        format++;
    }

    return count;
}

int printf(const char *format, ...) {
    va_list ap;
    va_start(ap, format);
    int result = vprintf_impl(format, ap);
    va_end(ap);
    return result;
}

int vsprintf(char *str, const char *format, va_list ap) {
    struct sprintf_ctx ctx = {str, (size_t)-1, 0};  // Use maximum size_t value
    int result = vsprintf_impl(&ctx, format, ap);
    
    // Null terminate the string
    if (str) {
        str[ctx.pos] = '\0';
    }
    
    return result;
}

int sprintf(char *str, const char *format, ...) {
    va_list ap;
    va_start(ap, format);
    int result = vsprintf(str, format, ap);
    va_end(ap);
    return result;
}

int vsnprintf(char *str, size_t size, const char *format, va_list ap) {
    if (size == 0) {
        // Still need to calculate how many characters would be written
        struct sprintf_ctx ctx = {NULL, 0, 0};
        return vsprintf_impl(&ctx, format, ap);
    }
    
    struct sprintf_ctx ctx = {str, size, 0};
    vsprintf_impl(&ctx, format, ap);
    
    // Null terminate the string
    if (str && size > 0) {
        if (ctx.pos >= size) {
            str[size - 1] = '\0';
        } else {
            str[ctx.pos] = '\0';
        }
    }
    
    // Return the number of characters that would have been written
    return (int)ctx.pos;
}

int snprintf(char *str, size_t size, const char *format, ...) {
    va_list ap;
    va_start(ap, format);
    int result = vsnprintf(str, size, format, ap);
    va_end(ap);
    return result;
}

int puts(const char *s) {
    if (s == NULL) {
        return -1; // EOF
    }
    puts_impl(s);
    putchar_impl('\n');
    return 0; // Success (non-negative value)
}

int putchar(int c) {
    putchar_impl((char)c);
    return c;
}

// ========== assert.h ==========

void halt(int code) {
    ctrap((unsigned char)code);
}

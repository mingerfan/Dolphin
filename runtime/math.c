// Math library functions needed by RISC-V GCC
// These are needed for division operations on 64-bit integers

// 64-bit division
long long __divdi3(long long a, long long b) {
    int sign = ((a < 0) ^ (b < 0)) ? -1 : 1;
    
    // Convert to positive
    unsigned long long ua = (a < 0) ? -a : a;
    unsigned long long ub = (b < 0) ? -b : b;
    
    // Simple division algorithm
    unsigned long long result = 0;
    while (ua >= ub) {
        ua -= ub;
        result++;
    }
    
    return sign < 0 ? -result : result;
}

// 64-bit modulo
long long __moddi3(long long a, long long b) {
    int sign = (a < 0) ? -1 : 1;
    
    // Convert to positive
    unsigned long long ua = (a < 0) ? -a : a;
    unsigned long long ub = (b < 0) ? -b : b;
    
    // Simple modulo algorithm
    while (ua >= ub) {
        ua -= ub;
    }
    
    return sign < 0 ? -ua : ua;
}

// 64-bit unsigned division
unsigned long long __udivdi3(unsigned long long a, unsigned long long b) {
    unsigned long long result = 0;
    while (a >= b) {
        a -= b;
        result++;
    }
    return result;
}

// 64-bit unsigned modulo
unsigned long long __umoddi3(unsigned long long a, unsigned long long b) {
    while (a >= b) {
        a -= b;
    }
    return a;
}

// 64-bit multiplication
long long __muldi3(long long a, long long b) {
    return a * b;  // This should work with basic multiplication
}

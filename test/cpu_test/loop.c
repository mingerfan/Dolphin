int main() {
    int sum = 0;
    for (int i = 1; i <= 5; i++) {
        sum += i;
    }
    
    // Verify final sum is 15 (1+2+3+4+5)
    if (sum != 15) {
        while(1); // Hang if incorrect
    }
    
    return sum;
}

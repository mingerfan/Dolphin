int main() {
    int a = 10;
    int b = 20;
    int sum = a + b;
    
    // Verify sum is correct
    if (sum != 30) {
        while(1); // Hang if incorrect
    }
    
    return sum;
}

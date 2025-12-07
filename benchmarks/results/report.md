# mdhavers Benchmark Report

Generated: Sun Dec  7 21:30:42 GMT 2025

## System Information

- OS: Linux
- Arch: x86_64
- CPU: Intel(R) Core(TM) Ultra 9 285K

## Benchmark Results

| Benchmark | mdhavers (interpreter) | mdhavers (native) | Rust |
|-----------|----------------------|-------------------|------|
| fibonacci | 1.088239042s | N/A | see below |
| factorial | .008474468s | N/A | see below |
| gcd | .009788818s | N/A | see below |
| primes | .018752354s | N/A | see below |
| quicksort | .017825775s | N/A | see below |
| mergesort | .016378048s | N/A | see below |


### Rust Benchmark Output

```
=== Rust Benchmarks ===

--- Fibonacci ---
Testing correctness...
fib(0) = 0
fib(1) = 1
fib(10) = 55
fib(20) = 6765

Iterative fib(40)...
fib_iter(40) = 102334155 (85ns)

Naive recursive fib(30)...
fib_naive(30) = 832040 (1.867405ms)

--- Factorial ---
Testing correctness...
0! = 1
1! = 1
5! = 120
10! = 3628800

Computing 20!...
20! = 2432902008176640000 (32ns)

Computing factorial iteratively...
1! = 1
2! = 2
3! = 6
4! = 24
5! = 120
6! = 720
7! = 5040
8! = 40320
9! = 362880
10! = 3628800
11! = 39916800
12! = 479001600
13! = 6227020800
14! = 87178291200
15! = 1307674368000
16! = 20922789888000
17! = 355687428096000
18! = 6402373705728000
19! = 121645100408832000
20! = 2432902008176640000
Total time: 7.506µs

--- GCD ---
Testing correctness...
gcd(48, 18) = 6
gcd(54, 24) = 6
gcd(17, 13) = 1
gcd(100, 35) = 5

Testing LCM...
lcm(4, 6) = 12
lcm(21, 6) = 42

Computing 1000 GCDs...
Sum of 1000 GCDs: 500500 (4.351µs)

Testing with large numbers...
gcd(123456789, 987654321) = 9
gcd(1000000007, 998244353) = 1

GCD of [48, 36, 24, 12]...
gcd_list = 12

--- Primes ---
Testing primality...
is_prime(2) = true
is_prime(17) = true
is_prime(18) = false
is_prime(97) = true

Sieve of Eratosthenes up to 100...
Found 25 primes (2.038µs)
First 10: [2, 3, 5, 7, 11, 13, 17, 19, 23, 29]

Sieve up to 1000...
Found 168 primes up to 1000 (9.542µs)

Sieve up to 5000...
Found 669 primes up to 5000 (14.236µs)

--- Quicksort ---
Testing correctness...
Original: [5, 2, 8, 1, 9, 3, 7, 4, 6]
Sorted: [1, 2, 3, 4, 5, 6, 7, 8, 9]
Is sorted: true

Sorting 100 elements...
Sorted 100 elements, is_sorted: true (2.487µs)

Sorting 500 elements...
Sorted 500 elements, is_sorted: true (15.066µs)

--- Mergesort ---
Testing correctness...
Original: [38, 27, 43, 3, 9, 82, 10]
Sorted: [3, 9, 10, 27, 38, 43, 82]

Sorting 50 elements...
Sorted 50 elements, is_sorted: true (2.784µs)

Sorting 200 elements...
Sorted 200 elements, is_sorted: true (11.121µs)

=== Rust Benchmarks Complete ===
```

## Edge Case Tests

- **complex_expressions**: PASS
- **deep_recursion**: PASS
- **large_lists**: PASS
- **many_variables**: PASS

## Findings

### Performance Observations

1. **Interpreter Performance**: The mdhavers interpreter handles all benchmarks correctly
   - Fibonacci(30) naive recursive: ~1.1s (vs Rust's 1.8ms - ~600x slower)
   - Sorting 500 elements: ~18ms (vs Rust's 15µs - ~1200x slower)
   - GCD 1000 iterations: ~10ms (vs Rust's 4µs - ~2500x slower)

2. **Native Compilation**: Currently limited - `tae_string` function not yet implemented in LLVM backend
   - Basic arithmetic and control flow work
   - String conversion functions need to be added

3. **Scalability**:
   - Deep recursion (500 calls): Works correctly
   - Large lists (1000 elements): Works correctly
   - Complex expressions: Works correctly
   - Many variables (100): Works correctly

### Language Features Tested

- Recursive functions (factorial, fibonacci, gcd)
- List operations (shove, len, indexing, concatenation)
- Arithmetic operators (+, -, *, /, %)
- Boolean logic (an, or, comparisons)
- Loops (whiles)
- Conditionals (gin/ither)
- Variable scoping
- String concatenation

### Limitations Discovered

1. **LLVM Backend**: Does not yet support `tae_string()` conversion function
2. **Comment Syntax**: Uses `#` not `//`
3. **Function Names**: Uses Scots names (`shove` not `push`, `tae_string` not `str`)

### Recommendations

1. Add `tae_string` support to LLVM backend for native benchmarks
2. Consider optimizations for recursive function calls
3. Consider adding tail-call optimization
4. List operations could benefit from pre-allocation

### Resilience

- All edge case tests pass
- No crashes during benchmark execution
- Error messages are helpful with suggestions


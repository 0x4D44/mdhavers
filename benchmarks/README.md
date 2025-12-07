# mdhavers Benchmark Suite

Comprehensive benchmarks comparing mdhavers interpreter performance against native Rust implementations.

## Quick Start

```bash
./run_benchmarks.sh
```

Results will be saved to `results/report.md`.

## Directory Structure

```
benchmarks/
├── run_benchmarks.sh     # Main benchmark runner
├── results/
│   └── report.md        # Generated benchmark report
├── mdhavers/            # mdhavers benchmark programs
│   ├── fibonacci.braw   # Recursive & iterative fibonacci
│   ├── factorial.braw   # Factorial computation
│   ├── gcd.braw         # Greatest common divisor
│   ├── primes.braw      # Sieve of Eratosthenes
│   ├── quicksort.braw   # Quicksort algorithm
│   └── mergesort.braw   # Mergesort algorithm
├── rust/                # Equivalent Rust implementations
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── fibonacci.rs
│       ├── factorial.rs
│       ├── gcd.rs
│       ├── primes.rs
│       ├── quicksort.rs
│       └── mergesort.rs
└── edge_cases/          # Edge case stress tests
    ├── deep_recursion.braw
    ├── large_lists.braw
    ├── complex_expressions.braw
    └── many_variables.braw
```

## Benchmarks

### Algorithm Benchmarks

| Benchmark | Description |
|-----------|-------------|
| Fibonacci | Naive recursive and iterative implementations |
| Factorial | Recursive factorial computation |
| GCD | Euclidean algorithm for greatest common divisor |
| Primes | Sieve of Eratosthenes |
| Quicksort | Recursive quicksort with list partitioning |
| Mergesort | Divide-and-conquer merge sort |

### Edge Case Tests

| Test | Description |
|------|-------------|
| Deep Recursion | Tests stack depth with 500+ recursive calls |
| Large Lists | Creates and manipulates 1000+ element lists |
| Complex Expressions | Deeply nested arithmetic and boolean expressions |
| Many Variables | Declares and uses 100+ variables |

## Running Individual Benchmarks

```bash
# Run a specific mdhavers benchmark
mdhavers run mdhavers/fibonacci.braw

# Run Rust benchmarks
cd rust && cargo run --release

# Run edge case tests
mdhavers run edge_cases/deep_recursion.braw
```

## Results Summary

The interpreter handles all benchmarks correctly with expected performance characteristics:

- **Recursion**: Handles deep recursion (500+ calls) without issues
- **Lists**: Correctly manages large lists (1000+ elements)
- **Expressions**: Properly evaluates complex nested expressions
- **Variables**: No issues with many variables in scope

See `results/report.md` for detailed timing comparisons.

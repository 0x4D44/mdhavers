# Golden Tests for mdhavers

Golden tests verify that mdhavers programs produce expected output. Each test consists of:
- A `.braw` source file
- A `.expected` file with the expected output

## Running Tests

```bash
# Run all golden tests
cargo test golden_tests --features llvm

# Run interpreter-only golden tests
cargo test golden_tests_interpreter

# Run native-only golden tests (requires LLVM)
cargo test golden_tests_native --features llvm
```

## Adding New Tests

1. Create a `.braw` file in the appropriate category directory
2. Run the program manually to verify output: `cargo run -- path/to/test.braw`
3. Create a `.expected` file with the exact expected output
4. Run `cargo test golden_tests` to verify

## Test Structure

```
tests/golden/
├── basics/          # Fundamental language features
├── control_flow/    # Conditionals, loops, match
├── functions/       # Functions and lambdas
├── data_structures/ # Lists, dicts, ranges
├── classes/         # OOP features
├── builtins/        # Built-in functions
├── algorithms/      # Complex programs
├── edge_cases/      # Corner cases
└── integration/     # Real-world scenarios
```

## Special Markers

Add these comments to tests for special handling:

- `// SKIP_NATIVE` - Skip for LLVM native compilation
- `// SKIP_INTERPRETER` - Skip for interpreter

## Updating Expectations

To update all expected files from current output:

```bash
./tests/golden/update_expectations.sh
```

**Review changes carefully before committing!**

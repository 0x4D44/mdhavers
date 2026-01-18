# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

mdhavers is a Scots programming language with multiple execution backends. The codebase consists of:
- **Interpreter**: Direct AST execution (main runtime)
- **JavaScript Compiler**: Transpiles to JS for browser/Node.js
- **WASM Compiler**: Generates WebAssembly Text format (WAT)
- **LLVM Backend**: Native code compilation (optional feature)
- **LSP Server**: Language server for editor integration

## Build Commands

### Basic Build and Test

```bash
# Build with auto-detected features (uses Makefile)
make build

# Build release (auto-detects LLVM)
make release

# Run all tests
make test
cargo test

# Run specific test
cargo test test_name

# Check for errors without building
cargo check
make check
```

### Feature-Specific Builds

```bash
# Default build includes ALL features (llvm, graphics, audio, native)
# Requires: ./scripts/setup-deps.sh (Linux) or .\scripts\setup-deps.ps1 (Windows)
cargo build

# Build with all features explicitly
cargo build --features full
make build-full

# Minimal build for CI or systems without optional dependencies
cargo build --no-default-features --features minimal
make build-minimal

# Build without any optional features (CLI only, no native networking)
cargo build --no-default-features --features cli

# Build with DTLS/SRTP support (requires OpenSSL, Unix only)
cargo build --features dtls

# Check LLVM detection status
make status

# Install dependencies first (platform-aware)
make setup
```

### Code Quality

```bash
# Format code
cargo fmt
make fmt

# Run clippy lints
cargo clippy -- -D warnings
make clippy

# Run coverage (requires cargo-llvm-cov)
make coverage                  # Show summary
make coverage-gate             # Fail if <98% line coverage
make coverage-guardrails       # Run with invariant checks
```

### Installation

```bash
# Install to cargo bin directory
make install

# Install to ~/.mdhavers with shell completions
make install-local

# Uninstall from ~/.mdhavers
make uninstall

# Create distribution tarball
make package
```

## Architecture

### Core Pipeline

1. **Lexer** (`src/lexer.rs`) - Tokenizes source using logos crate
2. **Parser** (`src/parser.rs`) - Builds AST from tokens
3. **Backends**:
   - **Interpreter** (`src/interpreter.rs`) - Direct AST execution
   - **JS Compiler** (`src/compiler.rs`) - Transpiles to JavaScript
   - **WASM Compiler** (`src/wasm_compiler.rs`) - Generates WAT
   - **LLVM Backend** (`src/llvm/`) - Native code via LLVM IR

### Key Modules

- **AST** (`src/ast.rs`) - Abstract syntax tree definitions
- **Value** (`src/value.rs`) - Runtime value representation (tagged union)
- **Error** (`src/error.rs`) - Error types with Scots-flavored messages
- **Token** (`src/token.rs`) - Token definitions for lexer
- **Formatter** (`src/formatter.rs`) - Code formatter
- **Audio** (`src/audio/`) - Unified audio abstraction with backend selection:
  - `mod.rs` - Public API and backend selection
  - `backend_miniaudio.rs` - miniaudio backend (standalone audio)
  - `backend_raylib.rs` - raylib backend (when graphics enabled)
  - `midi.rs` - MIDI synthesis via rustysynth
- **Graphics** (`src/graphics.rs`) - 2D graphics via raylib (optional)
- **Tri** (`src/tri.rs`) - 3D graphics module (optional)
- **Logging** (`src/logging.rs`) - Logging infrastructure
- **LSP** (`src/lsp/`) - Language server implementation

### LLVM Backend Structure

Located in `src/llvm/`:
- `mod.rs` - Module entry point
- `compiler.rs` - Main LLVM compiler driver
- `codegen.rs` - AST to LLVM IR code generation
- `builtins.rs` - Built-in function implementations
- `runtime.rs` - Runtime library interface
- `types.rs` - Type system and inference

### Runtime Libraries

Located in `runtime/`:
- **`mdh_runtime.c/.h`** - C runtime for LLVM-compiled code
  - String/list/dict operations
  - Memory management hooks (Boehm GC interface)
  - Built-in function implementations
- **`mdh_runtime_rs/`** - Rust runtime library
  - Provides regex, JSON, and other high-level operations
  - Linked as static library (`mdh_runtime_rs.a`)
- **`gc_stub.c/.o`** - GC stub for standalone builds
- **`js/`** - JavaScript runtime helpers for browser compilation
- **`mdh_rustysynth_wasm/`** - WASM MIDI synthesizer helper

Build runtime: `cd runtime && make`

### Test Organization

- **`tests/golden_tests.rs`** - Golden file tests (examples/)
- **`tests/backend_parity.rs`** - Cross-backend consistency tests
- **`tests/llvm_*.rs`** - LLVM-specific tests
- **`tests/interpreter_*.rs`** - Interpreter-specific tests
- **`tests/*_coverage*.rs`** - Coverage-focused tests for specific code paths

Golden tests use fixtures in `tests/golden/` and compare against expected outputs.

## Language Features

mdhavers uses Scots vocabulary for keywords:
- `ken` (know) - variable declaration
- `gin` (if) - conditional
- `dae` (do) - function definition
- `kin` (type/family) - class definition
- `blether` (chat) - print statement
- `gie` (give) - return statement
- `aye`/`nae` - true/false
- `naething` - null/nothing

See README.md for comprehensive language documentation.

## Development Patterns

### Adding Built-in Functions

1. **Interpreter**: Add case in `Interpreter::call_builtin()` in `src/interpreter.rs`
2. **JS Compiler**: Add runtime helper in `Compiler::emit_runtime()` in `src/compiler.rs`
3. **WASM Compiler**: Add import declaration in `src/wasm_compiler.rs`
4. **LLVM Backend**: Add function in `src/llvm/builtins.rs`
5. **Tests**: Add parity test in `tests/backend_parity.rs`

### Adding Language Features

1. Add token(s) to `src/token.rs` and lexer in `src/lexer.rs`
2. Update AST in `src/ast.rs`
3. Update parser in `src/parser.rs`
4. Implement in interpreter (`src/interpreter.rs`)
5. Implement in each compiler backend
6. Add tests in `tests/golden/` and backend-specific test files
7. Update formatter in `src/formatter.rs`
8. Update LSP in `src/lsp/`

### Coverage Requirements

This project maintains ≥98% line coverage. When adding code:
- Write comprehensive tests covering success and error paths
- Use `make coverage-gate` to verify coverage before committing
- Add `*_coverage.rs` test files for specific uncovered branches
- Never compromise production code structure just to hit coverage targets

## Common Commands

```bash
# Run a .braw file
./target/release/mdhavers examples/hello.braw
./target/release/mdhavers run examples/hello.braw

# Start REPL
./target/release/mdhavers repl

# Compile to JavaScript
./target/release/mdhavers compile examples/fizzbuzz.braw -o fizzbuzz.js
node fizzbuzz.js

# Compile to WAT
./target/release/mdhavers wasm examples/functions.braw -o functions.wat

# Format code
./target/release/mdhavers fmt examples/hello.braw
./target/release/mdhavers fmt examples/hello.braw --check

# Trace execution (debugger)
./target/release/mdhavers trace examples/hello.braw
./target/release/mdhavers trace examples/hello.braw -v  # verbose

# Check for errors
./target/release/mdhavers check examples/hello.braw

# Show tokens/AST (debugging)
./target/release/mdhavers tokens examples/hello.braw
./target/release/mdhavers ast examples/hello.braw
```

## Editor Integration

- **VS Code**: Extension in `editor/vscode/`
  - Build: `cd editor/vscode && npm install && npm run compile`
- **Vim/Neovim**: Syntax files in `editor/vim/`
- **LSP**: Binary at `target/release/mdhavers-lsp`
  - Provides diagnostics, hover, completion

## Dependencies

### Required
- Rust 1.70+ (edition 2021)
- Standard build tools (gcc/clang)

### Optional (auto-detected by Makefile)
- **LLVM 15** + `libzstd-dev` - For native compilation
- **OpenSSL** - For DTLS/SRTP support (dtls feature, Unix only)
- **raylib dependencies** - For graphics: `cmake`, `libx11-dev`, `libxrandr-dev`, `libxinerama-dev`, `libxcursor-dev`, `libxi-dev`, `libgl1-mesa-dev`
- **miniaudio** - For audio (no X11 needed for audio-only)
- **wasmtime**, **wat** - For WASM runner feature

## Project Structure

```
mdhavers/
├── src/               # Core source code
│   ├── main.rs        # CLI entry point
│   ├── lib.rs         # Library root
│   ├── lexer.rs       # Tokenizer
│   ├── parser.rs      # Parser
│   ├── ast.rs         # AST definitions
│   ├── interpreter.rs # Interpreter
│   ├── compiler.rs    # JS compiler
│   ├── wasm_compiler.rs # WASM compiler
│   ├── llvm/          # LLVM backend
│   └── lsp/           # Language server
├── runtime/           # Native runtime libraries
├── tests/             # Test suite
├── examples/          # Example .braw programs
├── stdlib/            # Standard library modules
├── playground/        # Web playground (WASM)
├── games/             # Game demos (Tetris)
├── editor/            # Editor integrations
├── installer/         # Installation scripts
├── scripts/           # Build/utility scripts
└── Makefile           # Primary build interface
```

## File Extensions

- `.braw` - mdhavers source files (Scots: "braw" = good/fine)
- `.wat` - WebAssembly Text format output
- `.wasm` - WebAssembly binary output
- `.js` - JavaScript compilation output

## Notes

- Default features: `full` (cli, native, llvm, graphics, audio) - run setup-deps script first
- Use `--no-default-features --features minimal` for CI builds without optional dependencies
- The `dtls` feature adds DTLS/SRTP support but requires OpenSSL and Unix
- The Makefile auto-detects LLVM and adjusts build accordingly
- Audio backend selection (unified audio abstraction):
  - `graphics` feature → raylib's built-in audio (no symbol conflicts)
  - `audio` feature alone → miniaudio backend
  - Both features → raylib audio takes precedence
- Most development uses the interpreter for quick iteration
- Use LLVM backend for production/performance-critical code
- Scots error messages are a core feature - preserve the dialect when modifying error handling

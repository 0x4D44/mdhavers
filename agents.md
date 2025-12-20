# Agents Guide

Quick orientation for contributors and automation working in the mdhavers repo.

## Project overview
- Language/runtime: Rust implementation of the mdhavers language (file extension `.braw`).
- Binaries: `mdhavers` CLI in `src/main.rs`, LSP server in `src/lsp/main.rs`.
- Library entry: `src/lib.rs` exposes parse, interpret, and compile helpers.

## Architecture map
- Front-end: `src/lexer.rs`, `src/parser.rs`, `src/ast.rs`.
- Runtime: `src/interpreter.rs`, `src/value.rs`, `src/error.rs`.
- Compilers: `src/compiler.rs` (JavaScript), `src/wasm_compiler.rs` (WAT), `src/llvm/` (LLVM, optional).
- Graphics: `src/graphics.rs` (raylib-backed, optional feature), `src/tri.rs` (three.js-style tri stub).
- WASM host runner: `src/wasm_runner.rs` (optional feature).
- Standard library: `stdlib/`.
- Examples and games: `examples/`, `games/`.

## Build and test
- Build/test (auto-detects LLVM): `make build`, `make release`, `make test`.
- Direct cargo: `cargo build`, `cargo test`.
- Format/lint: `make fmt`, `make clippy`.
- Feature flags:
  - `cli` (default) for CLI/LSP.
  - `llvm` (default) for LLVM backend.
  - `graphics` for raylib support.
  - `wasm_runner` for the built-in WAT/WASM host runner.
  - Example: `cargo build --no-default-features --features cli,graphics`.

## Common edit points
- New native functions: `src/interpreter.rs` (define/register natives).
- Graphics-native API: `src/graphics.rs` (registered in interpreter).
- tri module stub: `src/tri.rs` (interpreter-native), `stdlib/tri.braw` (fallback).
- CLI behavior: `src/main.rs`.
- Syntax/grammar changes: `src/lexer.rs`, `src/parser.rs`.
- Stdlib additions: `stdlib/` plus tests/examples as needed.

## Docs and tooling
- Main docs: `README.md`, extended docs in `docs/`.
- Playground: `playground/`.
- Editor tooling: `editor/`.

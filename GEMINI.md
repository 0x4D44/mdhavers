# GEMINI.md: Project Context for mdhavers

This document provides a comprehensive overview of the `mdhavers` project, designed to give context for development and interaction with the codebase.

## 1. Project Overview

`mdhavers` is the source code for a fully-featured, dynamically-typed programming language inspired by the Scots dialect. The language is implemented primarily in **Rust** and includes a rich set of tools and features. The file extension for the language is `.braw`.

### Core Components:

*   **Interpreter:** Executes `.braw` code directly.
*   **Compiler:** Compiles `.braw` code to multiple targets:
    *   JavaScript
    *   WebAssembly Text Format (`.wat`)
    *   LLVM (optional, for native compilation)
*   **REPL:** An interactive "Read-Eval-Print Loop" for live coding and experimentation.
*   **Language Server (LSP):** Provides editor support (diagnostics, autocomplete, hover info) for a modern IDE experience.
*   **Standard Library:** An extensive library of built-in functions, many with Scots-themed names (e.g., `blether` for print, `ken` for variable declaration).
*   **Web Playground:** A `wasm-pack` based web interface for trying the language in a browser.
*   **Example Code:** A large collection of `.braw` files in the `/examples` directory, demonstrating everything from basic syntax to a full Tetris game.

The project is structured as a Rust workspace with several key crates: the main `mdhavers` CLI, the `mdhavers-lsp` server, and the core library.

## 2. Building and Running

The project uses `cargo` for its primary build system, with a `Makefile` providing convenient high-level commands.

### Key Commands:

*   **Build (Debug):**
    ```bash
    make build
    # or
    cargo build
    ```

*   **Build (Release):**
    ```bash
    make release
    # or
    cargo build --release
    ```

*   **Run a `.braw` file:**
    ```bash
    ./target/release/mdhavers run examples/hello.braw
    ```

*   **Start the REPL:**
    ```bash
    ./target/release/mdhavers repl
    ```

*   **Compile a file:**
    ```bash
    # To JavaScript
    ./target/release/mdhavers compile examples/fizzbuzz.braw -o fizzbuzz.js

    # To WebAssembly Text Format
    ./target/release/mdhavers compile examples/functions.braw --target wat
    ```

*   **Run Tests:**
    ```bash
    make test
    # or
    cargo test
    ```

*   **Local Install:**
    The project provides an installer script to place binaries and configuration in `~/.mdhavers`.
    ```bash
    make install-local
    ```

## 3. Development Conventions

*   **Formatting:** The project uses `rustfmt` for code formatting. To format the entire project, run:
    ```bash
    make fmt
    # or
    cargo fmt
    ```

*   **Linting:** The project uses `clippy` for static analysis and linting. Run it with:
    ```bash
    make clippy
    # or
    cargo clippy -- -D warnings
    ```

*   **Testing:** Tests are written using Rust's built-in test framework and can be found throughout the `src` directory and in the `/tests` directory. The `pretty_assertions` crate is used for more readable test failure output.

*   **Language Design:** The language itself is heavily influenced by Scots dialect words, which are used for keywords, built-in functions, and even error messages. See the `README.md` for a complete guide to the language syntax and features.

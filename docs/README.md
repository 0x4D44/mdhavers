# mdhavers documentation

The documentation is an [mdBook](https://rust-lang.github.io/mdBook/) whose sources live in [`src/`](./src/). Every page below is readable straight from GitHub.

Bogle Castle (`games/bogle`) is a complete game written entirely in mdhavers on top of the screen and sound built-ins; it is a good worked example.

## Contents

- [Introduction](./src/index.md) - What mdhavers is, and a quick taste.

### Getting Started

- [Installation](./src/getting-started/installation.md) - Install the toolchain and build from source.
- [Hello World](./src/getting-started/hello-world.md) - Write and run your first program.
- [Editor Setup](./src/getting-started/editor-setup.md) - VS Code, Vim and language-server setup.
- [Running Programs](./src/getting-started/running-programs.md) - Run, REPL, compile and build commands in practice.

### Learning mdhavers

- [Basics](./src/learn/01-basics.md) - Variables, types and output.
- [Control Flow](./src/learn/02-control-flow.md) - Conditions and loops.
- [Functions](./src/learn/03-functions.md) - Defining and calling functions.
- [Data Structures](./src/learn/04-data-structures.md) - Lists, dictionaries and sets.
- [Strings](./src/learn/05-strings.md) - Working with strings.
- [Classes](./src/learn/06-classes.md) - Classes and inheritance.
- [Modules](./src/learn/07-modules.md) - Splitting code across files.
- [Error Handling](./src/learn/08-error-handling.md) - Catching and raising errors.
- [Pattern Matching](./src/learn/09-pattern-matching.md) - Matching values with `keek`.
- [Functional Programming](./src/learn/10-functional.md) - Lambdas, pipes and higher-order functions.

### Advanced Topics

- [Operator Overloading](./src/advanced/operator-overloading.md) - Custom behaviour for operators.
- [Destructuring](./src/advanced/destructuring.md) - Unpacking lists and dictionaries.
- [Performance Tips](./src/advanced/performance.md) - Making programs faster.
- [JavaScript Compilation](./src/advanced/interop.md) - Compiling to JavaScript.

### Reference

- [Keywords](./src/reference/keywords.md) - Every keyword and its English meaning.
- [Operators](./src/reference/operators.md) - Operators and precedence.
- [Built-in Functions](./src/reference/builtins.md) - Core built-in functions, with an index of the other built-in areas.
  - [Screen, Keyboard and Mouse](./src/reference/screen.md) - Framebuffer window, keyboard and mouse (`screen_*`, `key_*`, `mouse_*`).
  - [Sound (PCM output)](./src/reference/sound.md) - PCM sound output (`sound_*`).
  - [System, Files, Time and Logging](./src/reference/builtins/system.md) - Environment, process, file system, dates and logging.
  - [Text, Regex and JSON](./src/reference/builtins/text-data.md) - String helpers, regular expressions and JSON.
  - [Collections, Sets, Random and Maths](./src/reference/builtins/collections.md) - Sets, list helpers, random numbers and maths.
  - [Audio and MIDI](./src/reference/builtins/audio.md) - File-based audio and MIDI playback.
  - [Graphics (raylib)](./src/reference/builtins/graphics.md) - Immediate-mode 2D drawing (raylib).
  - [Tri 3D](./src/reference/builtins/tri.md) - The `tri` 3D module.
- [Standard Library](./src/reference/stdlib.md) - Standard library overview.
- [Standard Library modules](./src/stdlib/index.md) - One page per standard library module.
- [Examples](./src/reference/examples.md) - Guide to the programs in `examples/`.
- [CLI Reference](./src/reference/cli.md) - Command-line options.

### Appendix

- [Scots Glossary](./src/scots-glossary.md) - Scots words used by the language.

## Building the book

Install [mdBook](https://rust-lang.github.io/mdBook/guide/installation.html) (`cargo install mdbook`), then from the repository root:

```sh
mdbook build docs      # writes docs/book/
mdbook serve docs      # live preview at http://localhost:3000
```

## Keeping the built-in reference complete

`python3 scripts/docs_coverage.py` lists every built-in function registered in the Rust sources that no page documents, and exits non-zero if any are missing.


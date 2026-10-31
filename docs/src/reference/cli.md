# CLI Reference

Complete reference for the mdhavers command-line interface.

## Basic Usage

```bash
mdhavers [COMMAND] [OPTIONS] [FILE]
```

If no command is specified, mdhavers will:
- Run the file if a `.braw` file is provided
- Start the REPL if no file is provided

## Commands

### run

Run a `.braw` file.

```bash
mdhavers run program.braw
mdhavers program.braw  # Shorthand
```

### repl

Start the interactive Read-Eval-Print Loop.

```bash
mdhavers repl
mdhavers  # Shorthand (starts REPL when no file given)
```

#### REPL Commands

| Command | Description |
|---------|-------------|
| `help` | Show help message |
| `quit` | Exit the REPL |
| `haud yer wheesht` | Exit the REPL (Scottish way!) |
| `clear` | Clear the screen |
| `reset` | Reset interpreter state |
| `vars` / `env` | Show all defined variables |
| `wisdom` | Display a Scottish proverb |
| `examples` | Show example code snippets |
| `trace` | Toggle trace mode |
| `trace v` | Toggle verbose trace mode |

### compile

Compile mdhavers code to JavaScript.

```bash
# Print to stdout
mdhavers compile program.braw

# Write to file
mdhavers compile program.braw -o output.js
mdhavers compile program.braw --output output.js
```

**Options:**
- `-o, --output <FILE>`: Output file path

### build

Compile mdhavers code to a native executable (needs a build with the LLVM backend).

```bash
mdhavers build program.braw -o program
```

**Native checks happen at compile time.** The native backend refuses to build a
program that uses an undefined name or calls a builtin with the wrong number of
arguments, and reports the error then. The interpreter only raises these when
the line runs, so a program that does it on purpose and catches the error with
`hae_a_bash` works under `mdhavers run` but does not build natively. Example
programs that do this are marked `# SKIP_NATIVE`.

**Options:**
- `-o, --output <FILE>`: Output file path
- `-O, --opt-level <0-3>`: Optimisation level (default 2)
- `--emit-llvm`: Write LLVM IR (`.ll`) instead of building
- `--target <TARGET>`: `native` (the default) or `wasm32-emscripten` (also spelled
  `wasm32-unknown-emscripten`): the browser build, below
- `--emcc-arg <ARG>`: One extra argument for `emcc` at the browser build's link;
  repeat it for several

#### Building for the browser

```bash
mdhavers build --target wasm32-emscripten program.braw -o program.js
node program.js
```

The browser build compiles with the same LLVM backend for WebAssembly and links
with [Emscripten](https://emscripten.org)'s `emcc`, which must be on `PATH` (or
named by `EMCC`). It writes `program.js` and `program.wasm` beside it; load the
`.js` from a web page (with a `<canvas id="canvas">` if the program opens a
screen) or run it under `node`. The first build also compiles the runtime for
WebAssembly into a cache, which takes a minute; later builds reuse it until the
runtime sources or the Emscripten version change.

Pass Emscripten options with `--emcc-arg`. Two are common:

```bash
# Files the program reads, packed into program.data (path@path-in-the-program)
mdhavers build --target wasm32-emscripten game.braw -o game.js \
  --emcc-arg=--preload-file --emcc-arg=data@data

# JSPI: let a blocking loop's snooze suspend the program
mdhavers build --target wasm32-emscripten game.braw -o game.js --emcc-arg=-sJSPI
```

A web page cannot block, so in a page `snooze` raises an error unless the program
is built with `-sJSPI`. Draw a game's frames with
[`screen_loop`](./screen.md#screen_loopframe_fn) instead, which needs no JSPI.
Only browsers with WebAssembly JSPI can run a `-sJSPI` build; every build needs
WebAssembly exception handling and multi-value. Under `node`, `snooze` sleeps as
usual. The rest of the language, files read through `--preload-file`, the screen
(on a canvas) and sound (Web Audio) work as natively; see
[Screen](./screen.md) for what differs in the browser. Bogle Castle's page is
built this way: see `games/bogle/README.md`.

A browser build's memory grows as needed up to 2GB, its limit: the runtime keeps
every address below 2^31. `--emcc-arg=-sMAXIMUM_MEMORY=512MB` sets a lower limit;
a `MAXIMUM_MEMORY` or `INITIAL_MEMORY` above 2GB is refused.

### check

Check a file for syntax and semantic errors without running.

```bash
mdhavers check program.braw
```

Returns exit code 0 if no errors, non-zero otherwise.

### fmt

Format code to consistent style.

```bash
# Format and update file
mdhavers fmt program.braw

# Check formatting without modifying
mdhavers fmt program.braw --check
```

**Options:**
- `--check`: Check only, don't modify the file

### tokens

Display lexer tokens (debugging).

```bash
mdhavers tokens program.braw
```

**Example output:**
```
Token(Ken, "ken", line 1, col 1)
Token(Identifier("x"), "x", line 1, col 5)
Token(Equals, "=", line 1, col 7)
Token(Integer(42), "42", line 1, col 9)
```

### ast

Display the Abstract Syntax Tree (debugging).

```bash
mdhavers ast program.braw
```

### trace

Run with execution tracing.

```bash
# Basic trace - shows statements
mdhavers trace program.braw

# Verbose trace - shows values too
mdhavers trace program.braw -v
mdhavers trace program.braw --verbose
```

**Options:**
- `-v, --verbose`: Show detailed trace including values

**Example output:**
```
════════════════════════════════════════════════════════════
  🏴󠁧󠁢󠁳󠁣󠁴󠁿 mdhavers Tracer - Watchin' Yer Code Like a Hawk!
════════════════════════════════════════════════════════════

🏴󠁧󠁢󠁳󠁣󠁴󠁿 [line 1] ken x = ...
   → x is noo 42
🏴󠁧󠁢󠁳󠁣󠁴󠁿 [line 2] gin (if) statement
   → condition is aye
🏴󠁧󠁢󠁳󠁣󠁴󠁿 [line 2] takin' then branch
```

## Global Options

### --help / -h

Show help information.

```bash
mdhavers --help
mdhavers run --help
mdhavers compile --help
```

### --version

Show version information.

```bash
mdhavers --version
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Runtime error |
| 2 | Syntax error |
| 3 | File not found |

**Usage in scripts:**
```bash
mdhavers check program.braw
if [ $? -eq 0 ]; then
    echo "Code is valid!"
    mdhavers run program.braw
else
    echo "Found errors"
fi
```

## Environment Variables

The browser build (`build --target wasm32-emscripten`) reads:

| Variable | Meaning |
|---|---|
| `EMCC`, `EMAR` | The Emscripten compiler and archiver to run (default `emcc` and `emar` on `PATH`). |
| `MDH_RUNTIME_DIR` | Where the runtime sources are (default: the source tree's `runtime/`). |
| `MDH_WASM_RUNTIME_DIR` | Where the runtime built for WebAssembly is cached (default: `target/mdh-wasm-runtime` in the source tree). |

You can also pass data through stdin:

```bash
echo "input data" | mdhavers program.braw
```

## File Extensions

- `.braw` - Standard mdhavers source file (recommended)

The name "braw" is Scots for "good" or "fine".

## Examples

### Running Programs

```bash
# Run a simple program
mdhavers examples/hello.braw

# Run with trace
mdhavers trace examples/functions.braw -v
```

### Development Workflow

```bash
# Check for errors
mdhavers check myprogram.braw

# Format code
mdhavers fmt myprogram.braw

# Run if checks pass
mdhavers check myprogram.braw && mdhavers run myprogram.braw
```

### Building for Web

```bash
# Compile to JavaScript
mdhavers compile src/app.braw -o dist/app.js

# Minify (using external tool)
terser dist/app.js -o dist/app.min.js
```

### Interactive Development

```bash
# Start REPL for experimentation
mdhavers

mdhavers> ken x = 42
mdhavers> blether x * 2
84
mdhavers> trace
🏴󠁧󠁢󠁳󠁣󠁴󠁿 Trace mode ON
mdhavers[trace]> ken y = x + 10
🏴󠁧󠁢󠁳󠁣󠁴󠁿 [line 1] ken y = ...
   → y is noo 52
mdhavers[trace]> quit
```

### Debugging

```bash
# See how code is tokenized
mdhavers tokens myprogram.braw

# See the AST
mdhavers ast myprogram.braw

# Step through execution
mdhavers trace myprogram.braw -v
```

## LSP Server

mdhavers includes a Language Server Protocol implementation for IDE support.

```bash
# The LSP server binary
mdhavers-lsp

# Typically started by your editor automatically
# See Editor Setup for configuration
```

## Tips

1. **Use REPL for learning**: The interactive mode is great for experimenting

2. **Use trace for debugging**: When something's not working, trace shows exactly what's happening

3. **Format before committing**: Run `mdhavers fmt` to keep code consistent

4. **Check in CI/CD**: Use `mdhavers check` in your build pipeline

5. **Compile for production**: Use `mdhavers compile` to generate JavaScript for web deployment

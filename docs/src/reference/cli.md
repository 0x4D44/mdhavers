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

Currently, mdhavers doesn't use environment variables, but you can pass data through stdin:

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

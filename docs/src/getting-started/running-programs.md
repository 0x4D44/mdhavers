# Running Programs

Learn all the ways to run and work with mdhavers programs.

## Basic Execution

The simplest way to run a program:

```bash
mdhavers myprogram.braw
```

Or with the explicit `run` command:

```bash
mdhavers run myprogram.braw
```

Both are equivalent.

## Interactive REPL

Start the interactive Read-Eval-Print Loop:

```bash
mdhavers repl
# or just
mdhavers
```

### REPL Commands

| Command | Description |
|---------|-------------|
| `help` | Show available commands |
| `quit` | Exit the REPL |
| `haud yer wheesht` | Exit the REPL (Scottish way!) |
| `clear` | Clear the screen |
| `reset` | Reset interpreter state |
| `vars` / `env` | Show defined variables |
| `wisdom` | Get a Scottish proverb |
| `examples` | Show example code snippets |
| `trace` | Toggle trace mode |
| `trace v` | Toggle verbose trace mode |

### REPL Tips

- Multi-line input: Use `{` to start a block, the REPL will continue until you close it
- History: Use arrow keys to navigate previous commands
- Tab completion: Available for built-in function names

## Checking Code

Check for syntax and semantic errors without running:

```bash
mdhavers check myprogram.braw
```

This is useful for:
- Validating code before committing
- Integration with CI/CD pipelines
- Quick error checking in editors

## Formatting Code

Make your code look braw (good):

```bash
# Format and update the file
mdhavers fmt myprogram.braw

# Check formatting without modifying
mdhavers fmt myprogram.braw --check
```

The formatter ensures consistent:
- Indentation
- Spacing around operators
- Brace placement
- Line breaks

## Compiling to JavaScript

Convert mdhavers code to JavaScript for browser use:

```bash
# Print to stdout
mdhavers compile myprogram.braw

# Write to file
mdhavers compile myprogram.braw -o output.js
```

The generated JavaScript can run in any browser or Node.js environment.

## Debugging Tools

### Show Tokens

See how the lexer tokenizes your code:

```bash
mdhavers tokens myprogram.braw
```

Output shows each token with its type and position:
```
Token(Ken, "ken", line 1, col 1)
Token(Identifier("x"), "x", line 1, col 5)
Token(Equals, "=", line 1, col 7)
Token(Integer(42), "42", line 1, col 9)
```

### Show AST

View the Abstract Syntax Tree:

```bash
mdhavers ast myprogram.braw
```

Useful for understanding how your code is parsed.

### Trace Execution

Watch your program execute step-by-step:

```bash
# Basic trace
mdhavers trace myprogram.braw

# Verbose trace with values
mdhavers trace myprogram.braw -v
```

Example output:
```
════════════════════════════════════════════════════════════
  🏴󠁧󠁢󠁳󠁣󠁴󠁿 mdhavers Tracer - Watchin' Yer Code Like a Hawk!
════════════════════════════════════════════════════════════

🏴󠁧󠁢󠁳󠁣󠁴󠁿 [line 1] ken name = ...
   → name is noo "Hamish"
🏴󠁧󠁢󠁳󠁣󠁴󠁿 [line 2] gin (if) statement
   → condition is aye
🏴󠁧󠁢󠁳󠁣󠁴󠁿 [line 2] condition is aye - takin' then branch
```

The tracer shows:
- Variable declarations and their values
- Control flow decisions
- Function calls and returns
- Loop iterations
- Error handling

## Exit Codes

mdhavers uses standard exit codes:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Runtime error |
| 2 | Syntax error |
| 3 | File not found |

Use these in scripts:

```bash
mdhavers check myprogram.braw
if [ $? -eq 0 ]; then
    echo "Code is valid!"
else
    echo "Found errors"
fi
```

## Working with Multiple Files

### Modules

Import other `.braw` files with `fetch`:

```scots
# In main.braw
fetch "lib/helpers"
fetch "lib/math" tae m

# Use imported functions
helper_function()
m["square"](5)
```

Modules are resolved relative to the current file.

### Running from Different Directories

mdhavers resolves file paths relative to:
1. The current working directory
2. The directory containing the main script (for imports)

```bash
# Run from project root
mdhavers src/main.braw

# Imports in main.braw resolve relative to src/
```

## Environment Variables

mdhavers doesn't currently read environment variables directly, but you can pass them through input:

```bash
echo "$MY_VAR" | mdhavers myprogram.braw
```

Or use shell scripts to inject values.

## Command Reference

```
mdhavers [COMMAND] [OPTIONS] [FILE]

Commands:
  run      Run a .braw file (default)
  repl     Start interactive REPL
  compile  Compile to JavaScript
  check    Check for errors
  fmt      Format code
  tokens   Show lexer tokens
  ast      Show parsed AST
  trace    Trace execution

Options:
  -o, --output <FILE>   Output file (for compile)
  -v, --verbose         Verbose output (for trace)
  --check               Check only, don't modify (for fmt)
  -h, --help            Show help
  --version             Show version
```

## Next Steps

Now that you know how to run programs, start learning the [language basics](../learn/01-basics.md)!

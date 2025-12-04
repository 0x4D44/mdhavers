# mdhavers

**A Scots programming language - pure havers, but working havers!**

mdhavers is a dynamically-typed programming language that uses Scots vocabulary for its keywords and produces error messages in Scots dialect. It's a fully-featured language with an interpreter, a JavaScript compiler, and a friendly REPL.

## Quick Start

```bash
# Build the project
cargo build --release

# Run a .braw file
./target/release/mdhavers examples/hello.braw

# Start the interactive REPL
./target/release/mdhavers repl

# Compile to JavaScript
./target/release/mdhavers compile examples/fizzbuzz.braw
```

## Language Guide

### Variables

Use `ken` (meaning "to know") to declare variables:

```scots
ken name = "Hamish"
ken age = 42
ken pi = 3.14159
ken is_scottish = aye
ken is_english = nae
ken nothing = naething
```

### Data Types

- **Integers**: `42`, `-17`, `0`
- **Floats**: `3.14`, `-0.5`
- **Strings**: `"Hello, Scotland!"`
- **Booleans**: `aye` (true), `nae` (false)
- **Null**: `naething`
- **Lists**: `[1, 2, 3]`
- **Dictionaries**: `{"name": "Hamish", "age": 42}`

### Control Flow

#### If/Else (gin/ither)

```scots
gin temperature < 10 {
    blether "It's pure Baltic oot there!"
} ither gin temperature < 20 {
    blether "It's a wee bit nippy."
} ither {
    blether "Rare weather fer Scotland!"
}
```

#### While Loop (whiles)

```scots
ken count = 1
whiles count <= 5 {
    blether count
    count = count + 1
}
```

#### For Loop (fer)

```scots
# Loop over a range
fer i in 1..10 {
    blether i
}

# Loop over a list
ken cities = ["Edinburgh", "Glasgow", "Aberdeen"]
fer city in cities {
    blether city
}
```

#### Break and Continue

```scots
fer i in 1..100 {
    gin i > 10 {
        brak  # Break out of loop
    }
    gin i % 2 == 0 {
        haud  # Continue to next iteration
    }
    blether i
}
```

### Functions

Use `dae` (meaning "to do") to define functions, and `gie` (meaning "to give") to return values:

```scots
dae greet(name) {
    blether "Hullo, " + name + "!"
}

dae add(a, b) {
    gie a + b
}

dae factorial(n) {
    gin n <= 1 {
        gie 1
    }
    gie n * factorial(n - 1)
}

greet("Hamish")
blether add(5, 3)
blether factorial(5)
```

### Classes

Use `kin` (meaning "family/type") to define classes:

```scots
kin Animal {
    dae init(name, sound) {
        masel.name = name
        masel.sound = sound
    }

    dae speak() {
        blether masel.name + " says: " + masel.sound
    }
}

ken dug = Animal("Dug", "Woof!")
dug.speak()  # Output: Dug says: Woof!
```

### Input/Output

```scots
# Print output
blether "Hullo, World!"

# Get user input
ken name = speir "Whit's yer name? "
blether "Nice tae meet ye, " + name
```

### Built-in Functions

| Function | Description |
|----------|-------------|
| `len(x)` | Length of string, list, or dict |
| `whit_kind(x)` | Type of value |
| `tae_string(x)` | Convert to string |
| `tae_int(x)` | Convert to integer |
| `tae_float(x)` | Convert to float |
| `shove(list, x)` | Append to list |
| `yank(list)` | Pop from list |
| `keys(dict)` | Get dictionary keys |
| `values(dict)` | Get dictionary values |
| `sort(list)` | Sort a list |
| `reverse(x)` | Reverse list or string |
| `contains(x, y)` | Check if x contains y |
| `split(str, delim)` | Split string |
| `join(list, delim)` | Join list to string |
| `abs(n)` | Absolute value |
| `min(a, b)` | Minimum value |
| `max(a, b)` | Maximum value |
| `sqrt(n)` | Square root |
| `floor(n)` | Floor |
| `ceil(n)` | Ceiling |
| `round(n)` | Round |

## Keyword Reference

| Scots | English | Usage |
|-------|---------|-------|
| `ken` | know | Variable declaration |
| `gin` | if | Conditional |
| `ither` | other/else | Else clause |
| `whiles` | while | While loop |
| `fer` | for | For loop |
| `gie` | give | Return |
| `blether` | chat/talk | Print |
| `speir` | ask | Input |
| `dae` | do | Function definition |
| `kin` | family/type | Class definition |
| `thing` | thing | Struct definition |
| `aye` | yes | True |
| `nae` | no | False / Not |
| `naething` | nothing | Null |
| `an` | and | Logical AND |
| `or` | or | Logical OR |
| `brak` | break | Break from loop |
| `haud` | hold | Continue in loop |
| `masel` | myself | Self reference |
| `in` | in | For loop iteration |

## CLI Commands

```bash
# Run a file
mdhavers run program.braw
mdhavers program.braw  # shorthand

# Start REPL
mdhavers repl
mdhavers  # shorthand

# Compile to JavaScript
mdhavers compile program.braw
mdhavers compile program.braw -o output.js

# Check for errors
mdhavers check program.braw

# Show tokens (debug)
mdhavers tokens program.braw

# Show AST (debug)
mdhavers ast program.braw
```

## Error Messages

mdhavers gives you error messages in Scots:

```
Och! Ah dinnae ken whit 'xyz' is at line 5, column 3
Haud yer wheesht! Unexpected '}' at line 10 - ah wis expectin' ')'
Awa' an bile yer heid! 'foo' hasnae been defined yet at line 7
Ye numpty! Tryin' tae divide by zero at line 12
That's pure mince! Type error at line 8: Cannae add integer an' string
Yer bum's oot the windae! Function 'greet' expects 1 arguments but ye gave it 3
```

## Examples

See the `examples/` directory for sample programs:

- `hello.braw` - Hello World
- `variables.braw` - Variable types and operations
- `control_flow.braw` - If statements and loops
- `functions.braw` - Functions and recursion
- `classes.braw` - Object-oriented programming
- `fizzbuzz.braw` - Classic FizzBuzz (Scottish style!)
- `primes.braw` - Prime number finder
- `sorting.braw` - Sorting demonstrations

## Building from Source

```bash
# Clone the repository
git clone <repo-url>
cd mdhavers

# Build
cargo build --release

# Run tests
cargo test
```

## License

MIT

---

*"This is havers, but it's working havers!"*

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

#### Inheritance

Use `fae` (from) for class inheritance:

```scots
kin Animal {
    dae init(name) {
        masel.name = name
    }
    dae speak() {
        blether masel.name + " makes a noise"
    }
}

kin Dog fae Animal {
    dae speak() {
        blether masel.name + " says: Woof!"
    }
    dae wag() {
        blether masel.name + " wags tail!"
    }
}

ken rex = Dog("Rex")
rex.speak()  # Rex says: Woof!
rex.wag()    # Rex wags tail!
```

### Error Handling

Use `hae_a_bash` (have a bash/try) and `gin_it_gangs_wrang` (if it goes wrong/catch):

```scots
hae_a_bash {
    ken result = 10 / 0
    blether result
} gin_it_gangs_wrang err {
    blether "Caught error: " + err
}

# Nested error handling
hae_a_bash {
    blether "Trying something risky..."
    hae_a_bash {
        ken x = undefined_var
    } gin_it_gangs_wrang inner {
        blether "Inner catch: " + inner
    }
    blether "Continuing..."
} gin_it_gangs_wrang outer {
    blether "Outer catch: " + outer
}
```

### Pattern Matching

Use `keek` (peek/look at) and `whan` (when) for pattern matching:

```scots
ken day = 3

keek day {
    whan 1 -> { blether "Monday" }
    whan 2 -> { blether "Tuesday" }
    whan 3 -> { blether "Wednesday" }
    whan _ -> { blether "Another day" }
}

# Match with variable binding
ken mystery = 42
keek mystery {
    whan 0 -> { blether "Zero" }
    whan x -> { blether "Got: " + tae_string(x) }
}

# Match in a function
dae describe(n) {
    keek n {
        whan 0 -> { gie "naething" }
        whan 1 -> { gie "ane" }
        whan 2 -> { gie "twa" }
        whan _ -> { gie "mony" }
    }
    gie "unknown"
}
```

### Lambdas

Anonymous functions use pipe syntax:

```scots
ken add = |a, b| a + b
ken square = |x| x * x

blether add(3, 4)    # 7
blether square(5)    # 25
```

### Higher-Order Functions

Scots-named functional programming:

```scots
ken nums = [1, 2, 3, 4, 5]

# gaun - map (Scots: "going over")
ken doubled = gaun(nums, |x| x * 2)      # [2, 4, 6, 8, 10]

# sieve - filter
ken evens = sieve(nums, |x| x % 2 == 0)  # [2, 4]

# tumble - reduce/fold (Scots: "tumble together")
ken sum = tumble(nums, 0, |acc, x| acc + x)  # 15

# ilk - for-each (Scots: "each")
ilk(nums, print_func)
```

### F-Strings

Format strings with `{expression}` interpolation:

```scots
ken name = "Hamish"
ken age = 42
blether f"Hullo, {name}! Ye are {age} years auld."
blether f"Twa plus three is {2 + 3}"
blether f"Cities: {["Edinburgh", "Glasgow"]}"
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

**Scots-Flavored Functions** (the guid stuff!):

| Function | Scots Word Meaning | Description |
|----------|-------------------|-------------|
| `heid(x)` | head | Get first element |
| `bum(x)` | backside | Get last element |
| `tail(x)` | tail | All but first element |
| `scran(x, start, end)` | food/grab | Slice a portion |
| `slap(a, b)` | friendly slap | Concatenate lists/strings |
| `sumaw(list)` | sum all | Sum all numbers |
| `coont(x, y)` | count | Count occurrences |
| `wheesht(str)` | be quiet | Trim whitespace |
| `shuffle(list)` | - | Randomly shuffle |
| `jammy(min, max)` | lucky | Random integer in range |
| `the_noo()` | the now | Current timestamp |
| `clype(x)` | tell/snitch | Debug print with type |

**File I/O Functions**:

| Function | Description |
|----------|-------------|
| `scrieve(path, content)` | Write to file (Scots: write) |
| `read_file(path)` | Read entire file |
| `read_lines(path)` | Read file as list of lines |
| `append_file(path, content)` | Append to file |
| `file_exists(path)` | Check if file exists |

**Standard Functions**:

| Function | Description |
|----------|-------------|
| `len(x)` | Length of string, list, or dict |
| `whit_kind(x)` | Type of value |
| `tae_string(x)` | Convert to string |
| `tae_int(x)` | Convert to integer |
| `tae_float(x)` | Convert to float |
| `shove(list, x)` | Append to list (push) |
| `yank(list)` | Pop from list |
| `keys(dict)` | Get dictionary keys |
| `values(dict)` | Get dictionary values |
| `sort(list)` | Sort a list |
| `reverse(x)` | Reverse list or string |
| `contains(x, y)` | Check if x contains y |
| `split(str, delim)` | Split string |
| `join(list, delim)` | Join list to string |
| `upper(str)` | Convert to uppercase |
| `lower(str)` | Convert to lowercase |
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
| `hae_a_bash` | have a bash | Try block |
| `gin_it_gangs_wrang` | if it goes wrong | Catch block |
| `keek` | peek/look | Match statement |
| `whan` | when | Match case |

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
- `bubblesort.braw` - Bubblesort with index assignment
- `scots_stdlib.braw` - Scots-flavored standard library demo
- `try_catch.braw` - Error handling examples
- `match.braw` - Pattern matching examples
- `higher_order.braw` - Lambdas and higher-order functions
- `fstrings.braw` - F-string interpolation examples
- `inheritance.braw` - Class inheritance with `fae`
- `file_io.braw` - File I/O operations

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

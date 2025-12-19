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
./target/release/mdhavers compile examples/fizzbuzz.braw -o fizzbuzz.js
node fizzbuzz.js

# Compile to WebAssembly Text format
./target/release/mdhavers compile examples/functions.braw --target wat

# Try the web playground
cd playground && ./build.sh && cd web && python3 -m http.server 8080

# Play Tetris!
open games/tetris/index.html
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
- **Strings**: `"Hello, Scotland!"` or `'single quotes too'`
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

#### Default Parameters

Functions can hae default values fer parameters (staundart values):

```scots
# Parameter with default value
dae greet(name, greeting = "Hullo") {
    blether f"{greeting}, {name}!"
}

greet("Hamish")              # Uses default: "Hullo, Hamish!"
greet("Morag", "Och aye")    # Custom: "Och aye, Morag!"

# Multiple defaults
dae make_order(item, quantity = 1, price = 10) {
    gie quantity * price
}

make_order("haggis")           # 1 * 10 = 10
make_order("whisky", 3)        # 3 * 10 = 30
make_order("shortbread", 5, 2) # 5 * 2 = 10

# Defaults with lambda values
dae process(items, transform = |x| x) {
    gie gaun(items, transform)
}
```

Note: Parameters wi' defaults must come efter parameters wi'oot defaults.

### Destructuring

Unpack lists and strings intae individual variables:

```scots
# Basic destructuring
ken [a, b, c] = [1, 2, 3]
blether a  # 1

# With rest pattern (...) tae capture remaining elements
ken [first, ...rest] = [1, 2, 3, 4, 5]
blether first  # 1
blether rest   # [2, 3, 4, 5]

# Rest in the middle
ken [head, ...middle, tail] = [1, 2, 3, 4, 5]
blether middle  # [2, 3, 4]

# Ignore elements with _
ken [_, second, _] = ["skip", "take", "skip"]
blether second  # "take"

# Destructure strings intae characters
ken [a, b, c] = "ABC"
blether a  # "A"

# Practical: process function returns
dae get_bounds(list) {
    ken sorted = sort(list)
    gie [heid(sorted), bum(sorted)]
}
ken [min_val, max_val] = get_bounds([5, 2, 8, 1])
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

#### Operator Overloading

Define special methods tae customise how operators work wi' yer classes:

| Method | Operator | Scots Meaning |
|--------|----------|---------------|
| `__pit_thegither__(that)` | `+` | Put together (add) |
| `__tak_awa__(that)` | `-` | Take away (subtract) |
| `__times__(that)` | `*` | Multiply |
| `__pairt__(that)` | `/` | Divide |
| `__lave__(that)` | `%` | Remainder (what's left) |
| `__same_as__(that)` | `==` | Equal |
| `__differs_fae__(that)` | `!=` | Not equal |
| `__wee_er__(that)` | `<` | Smaller (less than) |
| `__wee_er_or_same__(that)` | `<=` | Smaller or same |
| `__muckle_er__(that)` | `>` | Bigger (greater than) |
| `__muckle_er_or_same__(that)` | `>=` | Bigger or same |

```scots
kin Vector {
    dae init(x, y) {
        masel.x = x
        masel.y = y
    }

    dae __pit_thegither__(that) {
        gie Vector(masel.x + that.x, masel.y + that.y)
    }

    dae __times__(scalar) {
        gie Vector(masel.x * scalar, masel.y * scalar)
    }
}

ken v1 = Vector(3, 4)
ken v2 = Vector(1, 2)
ken v3 = v1 + v2      # Vector(4, 6)
ken v4 = v1 * 2       # Vector(6, 8)
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

### Spread Operator

Use `...` tae skail (scatter/spread) lists and strings:

```scots
# Combine lists
ken a = [1, 2, 3]
ken b = [4, 5, 6]
ken combined = [...a, ...b]      # [1, 2, 3, 4, 5, 6]

# Insert elements
ken middle = [0, ...a, 99]       # [0, 1, 2, 3, 99]

# Spread in function calls
dae sum_three(x, y, z) {
    gie x + y + z
}
ken nums = [10, 20, 30]
blether sum_three(...nums)       # 60

# Spread strings intae characters
ken letters = [..."abc"]         # ["a", "b", "c"]
```

### Pipe Operator

Use `|>` fer fluent function chaining. The value on the left gets passed as the argument tae the function on the right:

```scots
# Basic pipe: value |> function
ken result = 5 |> |x| x * 2      # 10

# Chain multiple operations
dae add_one(x) { gie x + 1 }
dae triple(x) { gie x * 3 }

ken chained = 5 |> add_one |> triple |> add_one  # 19
# Same as: add_one(triple(add_one(5)))

# Works with built-in functions
ken text = "  hello  " |> wheesht |> upper  # "HELLO"

# Data processing pipeline
ken numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

dae get_evens(list) { gie sieve(list, |x| x % 2 == 0) }
dae double_all(list) { gie gaun(list, |x| x * 2) }

ken total = numbers |> get_evens |> double_all |> sumaw  # 60
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

### Assertions

Use `mak_siccar` (meaning "make sure" - famously said by Robert the Bruce!) for testing and validation:

```scots
ken x = 5
mak_siccar x == 5, "x should be 5"
mak_siccar x > 0   # without message

dae factorial(n) {
    mak_siccar n >= 0, "n must be non-negative"
    gin n <= 1 { gie 1 }
    gie n * factorial(n - 1)
}
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

### Modules

Use `fetch` to import other `.braw` files:

```scots
# Import a module (all exports available directly)
fetch "lib/maths"
blether square(5)  # 25

# Import with an alias (namespace)
fetch "lib/strings" tae str
blether str["capitalize"]("hello")  # "Hello"
```

Modules are resolved relative to the current file's directory. The `.braw` extension is optional.

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
| `drap(list, n)` | drop | Drop first n elements |
| `tak(list, n)` | take | Take first n elements |
| `grup(list, n)` | grip/group | Group into chunks of n |
| `pair_up(list)` | - | Create pairs from list |
| `fankle(a, b)` | tangle | Interleave two lists |
| `stoater(list)` | great one | Get best/max element |
| `braw(x)` | good/fine | Check if value is "good" |
| `clarty(x)` | dirty/messy | Check for duplicates |
| `dreich(str)` | dull/boring | Check if string is monotonous |
| `scottify(str)` | - | Convert English to Scots |
| `snooze(ms)` | - | Sleep for milliseconds |
| `indices_o(x, val)` | indices of | Find all indices of value |
| `braw_date(ts)` | braw date | Format date in Scottish style |
| `grup_up(list, fn)` | group up | Group elements by function |
| `pairt_by(list, fn)` | part by | Partition by predicate |
| `haverin(x)` | talking havers | Check if value is empty/nonsense |
| `scunner(x)` | disgusting | Check if value is negative/empty |
| `bonnie(x)` | pretty | Decorate value: "~~~ x ~~~" |
| `is_wee(x)` | is wee | Check if value is small |
| `is_muckle(x)` | is big | Check if value is large |
| `crabbit(n)` | grumpy | Check if number is negative |
| `roar(str)` | shout | Uppercase with "!": shout |
| `cannie(x)` | careful | Check if value is safe/valid |
| `glaikit(x)` | silly | Check if value is empty/zero/invalid |
| `wrang_sort(x, type)` | wrong sort | Check if wrong type |
| `tattie_scone(s, n)` | potato scone | Repeat string with \| separator |
| `haggis_hunt(s, needle)` | haggis hunt | Find all occurrences of substring |
| `sporran_fill(s, w, c)` | sporran fill | Center-pad string |
| `blether_format(s, d)` | format | Format string with dict placeholders |
| `ceilidh(l1, l2)` | dance | Interleave two lists like dancers |
| `dram(list)` | wee drink | Get random element from list |
| `birl(list, n)` | spin | Rotate list by n positions |
| `stooshie(str)` | chaos | Shuffle string characters |
| `clype(x)` | tell tales | Get debug info about a value |
| `sclaff(list)` | hit flat | Fully flatten nested lists |

**Scottish Exclamation Functions**:

| Function | Description |
|----------|-------------|
| `och(msg)` | Express disappointment: "Och! {msg}" |
| `jings(msg)` | Express surprise: "Jings! {msg}" |
| `crivvens(msg)` | Express astonishment: "Crivvens! {msg}" |
| `help_ma_boab(msg)` | Express extreme surprise: "Help ma boab! {msg}" |
| `roar(str)` | Shout: uppercase with "!" |
| `mutter(str)` | Whisper: "...{lowercase}..." |
| `blooter(str)` | Scramble string randomly |
| `numpty_check(x)` | Validate input with Scots feedback |

**Timing/Benchmarking Functions**:

| Function | Description |
|----------|-------------|
| `noo()` | Current timestamp in milliseconds ("now") |
| `tick()` | High-precision timestamp in nanoseconds |
| `bide(ms)` | Sleep for milliseconds ("bide" = wait) |

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
| `tae_bool(x)` | Convert to boolean |
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
| `pad_left(s, w, c)` | Pad string on left |
| `pad_right(s, w, c)` | Pad string on right |
| `is_a(x, type)` | Type checking |

**Math Functions**:

| Function | Description |
|----------|-------------|
| `abs(n)` | Absolute value |
| `min(a, b)` | Minimum value |
| `max(a, b)` | Maximum value |
| `sqrt(n)` | Square root |
| `floor(n)` | Floor |
| `ceil(n)` | Ceiling |
| `round(n)` | Round |
| `pooer(x, y)` | Power/exponent (x^y) |
| `sin(n)` | Sine (radians) |
| `cos(n)` | Cosine (radians) |
| `tan(n)` | Tangent (radians) |
| `asin(n)` | Arc sine |
| `acos(n)` | Arc cosine |
| `atan(n)` | Arc tangent |
| `atan2(y, x)` | Two-argument arc tangent |
| `hypot(x, y)` | Hypotenuse (sqrt(x² + y²)) |
| `log(n)` | Natural logarithm |
| `log10(n)` | Base 10 logarithm |
| `exp(n)` | e raised to the power |
| `degrees(rad)` | Convert radians to degrees |
| `radians(deg)` | Convert degrees to radians |
| `sign(n)` | Sign of number (-1, 0, or 1) |
| `clamp(n, min, max)` | Constrain value between min and max |
| `lerp(a, b, t)` | Linear interpolation |
| `gcd(a, b)` | Greatest common divisor |
| `lcm(a, b)` | Least common multiple |
| `factorial(n)` | Calculate factorial (max 20) |
| `is_even(n)` | Check if number is even |
| `is_odd(n)` | Check if number is odd |
| `is_prime(n)` | Check if number is prime |
| `PI` | Pi constant (3.14159...) |
| `E` | Euler's number (2.71828...) |
| `TAU` | Tau constant (2π) |

**Bitwise Operations**:

| Function | Description |
|----------|-------------|
| `bit_an(a, b)` | Bitwise AND |
| `bit_or(a, b)` | Bitwise OR |
| `bit_xor(a, b)` | Bitwise XOR |
| `bit_nae(n)` | Bitwise NOT |
| `bit_shove_left(n, shift)` | Left shift |
| `bit_shove_right(n, shift)` | Right shift |
| `bit_coont(n)` | Count set bits (popcount) |
| `tae_binary(n)` | Convert to binary string |
| `tae_hex(n)` | Convert to hexadecimal string |
| `tae_octal(n)` | Convert to octal string |
| `fae_binary(s)` | Parse binary string to integer |
| `fae_hex(s)` | Parse hex string to integer |

**Dictionary Functions**:

| Function | Description |
|----------|-------------|
| `dict_merge(d1, d2)` | Merge two dictionaries |
| `dict_get(d, key, default)` | Get value with default |
| `dict_has(d, key)` | Check if key exists |
| `dict_remove(d, key)` | Remove key from dictionary |
| `dict_invert(d)` | Swap keys and values |
| `items(d)` | Get list of [key, value] pairs |
| `fae_pairs(list)` | Create dict from pairs |

**List Statistics**:

| Function | Description |
|----------|-------------|
| `average(list)` | Calculate mean |
| `median(list)` | Calculate median |
| `product(list)` | Multiply all numbers |
| `minaw(list)` | Find minimum in list |
| `maxaw(list)` | Find maximum in list |
| `range_o(list)` | Calculate range (max - min) |

**Assertion Functions**:

| Function | Description |
|----------|-------------|
| `assert(cond, msg)` | Assert condition is true |
| `assert_equal(a, b)` | Assert two values are equal |
| `assert_nae_equal(a, b)` | Assert two values are not equal |
| `mak_siccar(cond, msg)` | Assert (like Robert the Bruce!) |

**More String Functions**:

| Function | Description |
|----------|-------------|
| `center(s, width, fill)` | Center string in field |
| `is_upper(s)` | Check if all uppercase |
| `is_lower(s)` | Check if all lowercase |
| `swapcase(s)` | Swap case of letters |
| `strip_left(s, chars)` | Strip leading characters |
| `strip_right(s, chars)` | Strip trailing characters |
| `replace_first(s, from, to)` | Replace first occurrence |
| `substr_between(s, start, end)` | Get substring between markers |

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
| `fetch` | fetch | Import module |
| `tae` | to | Module alias (fetch ... tae name) |
| `mak_siccar` | make sure | Assert (like Robert the Bruce!) |
| `...` | skail (scatter) | Spread operator fer lists |
| `\|>` | pipe | Pipe operator fer chaining |

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

# Compile to WebAssembly Text format (WAT)
mdhavers compile program.braw --target wat
mdhavers compile program.braw --target wat -o output.wat

# Check for errors
mdhavers check program.braw

# Format code (makes it look braw!)
mdhavers fmt program.braw
mdhavers fmt program.braw --check  # check only, dinnae modify

# Show tokens (debug)
mdhavers tokens program.braw

# Show AST (debug)
mdhavers ast program.braw

# Trace execution (step-by-step with Scottish commentary!)
mdhavers trace program.braw        # statements only
mdhavers trace program.braw -v     # verbose mode (shows values too)
```

## Compilation Targets

mdhavers can compile yer code tae multiple targets fer running in different environments.

### JavaScript Compilation

Compile mdhavers code tae JavaScript fer running in browsers or Node.js:

```bash
# Compile to JavaScript
mdhavers compile fizzbuzz.braw -o fizzbuzz.js

# Run with Node.js
node fizzbuzz.js
```

**Example - FizzBuzz in mdhavers:**

```scots
fer i in 1..101 {
    gin i % 15 == 0 {
        blether "FizzBuzz"
    } ither gin i % 3 == 0 {
        blether "Fizz"
    } ither gin i % 5 == 0 {
        blether "Buzz"
    } ither {
        blether i
    }
}
```

**Compiled JavaScript output:**

```javascript
for (let i = 1; i < 101; i++) {
    if ((i % 15) === 0) {
        console.log("FizzBuzz");
    } else if ((i % 3) === 0) {
        console.log("Fizz");
    } else if ((i % 5) === 0) {
        console.log("Buzz");
    } else {
        console.log(i);
    }
}
```

### WebAssembly Text Format (WAT)

Compile tae WAT fer high-performance execution:

```bash
# Compile to WAT
mdhavers compile maths.braw --target wat -o maths.wat

# Convert WAT to WASM using wat2wasm (from wabt toolkit)
wat2wasm maths.wat -o maths.wasm
```

**Example - Simple maths function:**

```scots
dae add(a, b) {
    gie a + b
}
```

**Compiled WAT output:**

```wat
(module
  (func $add (param $a i64) (param $b i64) (result i64)
    (i64.add
      (local.get $a)
      (local.get $b)))
  (export "add" (func $add)))
```

### Using the Rust Library

You can also compile programmatically using mdhavers as a library:

```rust
use mdhavers::{compile_to_js, compile_to_wat};

fn main() {
    let source = r#"
        dae greet(name) {
            blether "Hello, " + name + "!"
        }
        greet("World")
    "#;

    // Compile to JavaScript
    let js = compile_to_js(source).unwrap();
    println!("JavaScript:\n{}", js);

    // Compile to WAT
    let wat = compile_to_wat(source).unwrap();
    println!("WAT:\n{}", wat);
}
```

## Interactive Playground

mdhavers includes a web-based playground fer experimenting with code directly in yer browser.

### Features

- **Live Code Execution**: Run mdhavers code client-side using WebAssembly
- **Syntax Highlighting**: Beautiful dark theme with JetBrains Mono font
- **Code Formatting**: Auto-format yer code with one click
- **JavaScript Compilation**: View the compiled JavaScript output
- **Example Code**: Built-in examples covering all language features
- **Share Links**: Share yer code via URL

### Running the Playground Locally

```bash
# Navigate to the playground directory
cd playground

# Build the WASM module (requires wasm-pack)
./build.sh

# Or manually:
wasm-pack build --target web
cp -r pkg web/

# Serve the playground
cd web
python3 -m http.server 8080

# Open http://localhost:8080 in your browser
```

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Enter` | Run code |
| `Ctrl+Shift+F` | Format code |
| `Tab` | Insert 4 spaces |
| `Escape` | Close modal |

### Deployment

The `playground/web/` directory contains everything needed fer static hosting (GitHub Pages, Netlify, Vercel, etc.).

## Games

### Tetris

A fully-featured Tetris implementation written in mdhavers! The game demonstrates classes, control flow, and complex game logic.

#### Playing Tetris

**Browser Version (recommended):**
```bash
# Open the game directly in your browser
open games/tetris/index.html

# Or serve it locally
cd games/tetris
python3 -m http.server 8080
# Visit http://localhost:8080
```

**Controls:**

| Key | Action |
|-----|--------|
| `←` / `→` | Move piece left/right |
| `↓` | Soft drop (move down faster) |
| `↑` | Rotate piece |
| `Space` | Hard drop (instant drop) |
| `P` | Pause/Resume |

On mobile devices, use the on-screen buttons.

#### Game Features

- Classic Tetris gameplay with all 7 tetromino pieces
- Scottish-themed colours:
  - **I** (Turquoise) - Like the Scottish sea
  - **O** (Gold) - Like whisky
  - **T** (Purple) - Like heather
  - **S** (Green) - Like the Highlands
  - **Z** (Orange-red) - Like a sunset
  - **J** (Royal blue) - Like the Saltire
  - **L** (Orange) - Like Irn-Bru
- Ghost piece showing where yer piece will land
- Wall kicks fer rotation near edges
- Level progression (speeds up every 10 lines)
- Scoring system:
  - 1 line: 100 points × level
  - 2 lines: 300 points × level
  - 3 lines: 500 points × level
  - 4 lines (Tetris!): 800 points × level
  - Hard drop bonus: 2 points per cell

#### The mdhavers Source Code

The game logic is written entirely in mdhavers (`games/tetris/tetris.braw`):

```scots
# Tetris piece definitions
ken SHAPES = {
    "I": [[0, 0], [0, 1], [0, 2], [0, 3]],
    "O": [[0, 0], [0, 1], [1, 0], [1, 1]],
    "T": [[0, 0], [0, 1], [0, 2], [1, 1]],
    # ... more shapes
}

# Game class
kin TetrisGame {
    dae init() {
        masel.board = masel.create_empty_board()
        masel.score = 0
        masel.level = 1
        masel.spawn_piece()
    }

    dae move_down() {
        gin nae masel.check_collision(1, 0) {
            masel.current_row = masel.current_row + 1
        } ither {
            masel.lock_piece()
            masel.clear_lines()
            masel.spawn_piece()
        }
    }

    # ... more game logic
}
```

## Debugger/Tracer

mdhavers includes a tracer mode that shows step-by-step execution with Scottish commentary. It's pure dead brilliant for debugging!

```bash
# Basic trace - shows statements as they execute
mdhavers trace program.braw

# Verbose trace - shows values and more detail
mdhavers trace program.braw -v
```

Example output:
```
════════════════════════════════════════════════════════════
  🏴󠁧󠁢󠁳󠁣󠁴󠁿 mdhavers Tracer - Watchin' Yer Code Like a Hawk!
  Mode: Verbose (showin' everything)
════════════════════════════════════════════════════════════

🏴󠁧󠁢󠁳󠁣󠁴󠁿 [line 5] ken name = ...
   → name is noo Hamish
🏴󠁧󠁢󠁳󠁣󠁴󠁿 [line 6] gin (if) statement
   → condition is aye
🏴󠁧󠁢󠁳󠁣󠁴󠁿 [line 6] condition is aye - takin' then branch
🏴󠁧󠁢󠁳󠁣󠁴󠁿 [line 6] enterin' block
🏴󠁧󠁢󠁳󠁣󠁴󠁿   [line 7] blether (print): Welcome!
🏴󠁧󠁢󠁳󠁣󠁴󠁿 [line 6] leavin' block
🏴󠁧󠁢󠁳󠁣󠁴󠁿 [line 10] fer (for) loop: i in ...
   → iteratin' ower 3 items
   → iteration 1: i = 1
🏴󠁧󠁢󠁳󠁣󠁴󠁿 [line 10] fer loop done after 3 iterations

════════════════════════════════════════════════════════════
  🏴󠁧󠁢󠁳󠁣󠁴󠁿 Trace complete - Pure dead brilliant!
════════════════════════════════════════════════════════════
```

The tracer shows:
- Variable declarations with their values
- Control flow (if/gin, loops with iteration counts)
- Function calls and returns
- Try/catch blocks with error details
- Pattern matching with which arm matched

## Interactive REPL

The REPL (Read-Eval-Print Loop) provides an interactive environment for experimenting with mdhavers:

```bash
mdhavers repl    # Start the REPL
mdhavers         # Also starts REPL if no file given
```

### REPL Commands

| Command | Description |
|---------|-------------|
| `help` | Show help message |
| `quit` / `haud yer wheesht` | Exit the REPL |
| `clear` | Clear the screen |
| `reset` | Reset interpreter (clear all variables) |
| `wisdom` | Get a Scottish proverb |
| `examples` | Show example code snippets |
| `trace` | Toggle trace mode (see execution step-by-step) |
| `trace v` | Toggle verbose trace mode |
| `vars` / `env` | Show all defined variables |

### REPL Tracing

You can enable tracing directly in the REPL to debug code interactively:

```
mdhavers> trace
🏴󠁧󠁢󠁳󠁣󠁴󠁿 Trace mode ON - watchin' yer code like a hawk!

mdhavers[trace]> ken x = 42
🏴󠁧󠁢󠁳󠁣󠁴󠁿 [line 1] ken x = ...

mdhavers[trace]> trace
Trace mode OFF - back tae normal.
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

### Core Examples

- `hello.braw` - Hello World
- `variables.braw` - Variable types and operations
- `control_flow.braw` - If statements and loops
- `functions.braw` - Functions and recursion
- `classes.braw` - Object-oriented programming
- `fizzbuzz.braw` - Classic FizzBuzz (Scottish style!)
- `primes.braw` - Prime number finder
- `sorting.braw` - Sorting demonstrations
- `bubblesort.braw` - Bubblesort with index assignment

### Advanced Features

- `scots_stdlib.braw` - Scots-flavored standard library demo
- `try_catch.braw` - Error handling examples
- `match.braw` - Pattern matching examples
- `higher_order.braw` - Lambdas and higher-order functions
- `fstrings.braw` - F-string interpolation examples
- `inheritance.braw` - Class inheritance with `fae`
- `operator_overload.braw` - Operator overloading with classes
- `destructure.braw` - Destructuring assignment examples
- `spread.braw` - Spread operator (...) examples
- `pipe.braw` - Pipe operator (|>) examples
- `defaults.braw` - Default parameter values (staundart values)

### Fun Examples

- `scottish_pub.braw` - A wee Scottish pub simulation (classes, dicts, HOF)
- `ceilidh.braw` - Scottish dance party (math functions, lists, shuffling)
- `scots_words.braw` - Scottish vocabulary functions demo
- `scots_fun.braw` - New Scots vocabulary functions demo

### Utility Examples

- `file_io.braw` - File I/O operations
- `modules_demo.braw` - Demonstrating the module import system
- `assert_demo.braw` - Assertions with mak_siccar
- `test_example.braw` - Testing library demonstration
- `prelude_demo.braw` - Auto-loaded prelude functions demo
- `prelude_showcase.braw` - Demo of prelude functions (greetings, debug, validation)
- `new_functions.braw` - New higher-order functions demo
- `trace_demo.braw` - Demo file for the tracer (try `mdhavers trace`)
- `benchmark.braw` - Timing and benchmarking demo (noo, tick, bide)

### Standard Library

- `lib/maths.braw` - Mathematics utility library
- `lib/strings.braw` - String manipulation library
- `lib/collections.braw` - Data structures (stacks, queues, sets)
- `lib/functional.braw` - Functional programming utilities
- `lib/testing.braw` - Testing framework with assertions

### Games

- `games/tetris/tetris.braw` - Full Tetris game implementation

### Compilation Examples

You can compile any example tae JavaScript:

```bash
# Compile FizzBuzz to JavaScript
mdhavers compile examples/fizzbuzz.braw -o fizzbuzz.js
node fizzbuzz.js

# Compile to WAT
mdhavers compile examples/functions.braw --target wat -o functions.wat
```

Try it in the playground at `playground/web/` tae see live compilation!

## Building from Source

```bash
# Clone the repository
git clone <repo-url>
cd mdhavers

# Build minimal (interpreter only, no LLVM/graphics/audio)
cargo build --release --no-default-features --features cli

# Run tests
cargo test
```

### Building with LLVM Support

To enable native code compilation via LLVM, you need to install LLVM 15 and its dependencies:

**Ubuntu/Debian:**
```bash
# Install LLVM 15 and required libraries
sudo apt install llvm-15 llvm-15-dev libpolly-15-dev libzstd-dev

# Set the LLVM prefix environment variable
export LLVM_SYS_150_PREFIX=/usr/lib/llvm-15

# Add to your shell config to make it permanent
echo 'export LLVM_SYS_150_PREFIX=/usr/lib/llvm-15' >> ~/.bashrc
```

**Then build with LLVM:**
```bash
cargo build --release
# or explicitly:
cargo build --release --features llvm
```

**Verify LLVM detection (using the Makefile):**
```bash
make status
```

**Note:** Default features now enable `cli`, `llvm`, `graphics`, and `audio`.
If you want LLVM without graphics/audio:
```bash
cargo build --release --no-default-features --features cli,llvm
```

To build without LLVM (and without graphics/audio):
```bash
cargo build --release --no-default-features --features cli
```

### Audio (Soond)

Audio is enabled by default and independent of graphics. If you want to enable
audio explicitly (for example, without graphics), build with:

```bash
cargo build --release --no-default-features --features cli,llvm,audio
```

**Note:** Audio and graphics use raylib. On Ubuntu/WSL you’ll need:
```bash
sudo apt install cmake libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev
```
If you don’t have those, build with `--no-default-features` and add only what you need.

**Backend support:** Interpreter, LLVM/native, JavaScript, and WAT/WASM.

For JavaScript/WASM, audio uses WebAudio and a small rustysynth WASM helper.
You must host the following assets alongside your compiled output (or set overrides):
- `assets/wasm/mdh_rustysynth.wasm`
- `assets/soundfonts/MuseScore_General.sf2`

Optional overrides (set before running audio code):
```js
globalThis.__havers_audio_base = "/static/"; // prefix for audio assets
globalThis.__havers_soundfont = "/static/sf2/custom.sf2";
globalThis.__havers_midi_wasm = "/static/wasm/mdh_rustysynth.wasm";
```

For WAT/WASM in the browser, wire audio imports via the helper runtime:
```js
// Load the audio runtime + WASM host helpers first.
import "./runtime/js/audio_runtime.js";
import "./runtime/js/wasm_audio_host.js";

const imports = {
  env: {
    memory,
    // print_i32/print_f64/print_str, etc.
    ...mdh_wasm_audio_imports(memory),
  },
};
```

### Troubleshooting Raylib Builds

If you see an error like:
```
RandR headers not found; install libxrandr development package
```
install the X11 dev packages:
```bash
sudo apt install cmake libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev
```

If you hit:
```
Unable to find libclang
```
install clang + libclang and retry:
```bash
sudo apt install clang libclang-dev llvm-dev
export LIBCLANG_PATH=$(llvm-config --libdir)
```

If you want to avoid raylib entirely, build without graphics/audio:
```bash
cargo build --release --no-default-features --features cli,llvm
```

**Quick example:**
```scots
soond_stairt()
ken ding = soond_lade("assets/audio/ding.wav")
soond_spiel(ding)
```
Use `soond_ready(handle)` to check SFX load status on web backends.

**Streaming (MP3 + MIDI) needs updates:**
```scots
ken tune = muisic_lade("assets/audio/theme.mp3")
muisic_spiel(tune)

whiles aye {
    soond_haud_gang()  # keep streams flowing
}
```

MIDI uses a bundled default SoundFont at `assets/soundfonts/MuseScore_General.sf2` when you pass `naething` as the soundfont path:

```scots
ken song = midi_lade("assets/audio/wee_tune.mid", naething)
midi_spiel(song)
```

## Editor Support

mdhavers includes a **Language Server Protocol (LSP)** implementation for rich editor features:

- Real-time error diagnostics
- Hover documentation for keywords and built-ins
- Auto-completion with Scottish-flavored suggestions
- Syntax highlighting

### Installing the LSP Server

```bash
# Build the LSP server
cargo build --release

# The binary will be at target/release/mdhavers-lsp
# Add it to your PATH or configure your editor to find it
```

### VS Code (Full LSP Support)

1. Copy the `editor/vscode` folder to your VS Code extensions directory
2. Install dependencies: `cd editor/vscode && npm install && npm run compile`
3. Reload VS Code
4. The extension will automatically start the LSP server

Configuration options in VS Code settings:
- `mdhavers.lsp.path` - Path to mdhavers-lsp executable (default: "mdhavers-lsp")
- `mdhavers.lsp.enable` - Enable/disable the language server (default: true)

### Vim/Neovim (Syntax Highlighting + Optional LSP)

Add to your vim config:

```vim
" Add to your .vimrc or init.vim
au BufNewFile,BufRead *.braw set filetype=mdhavers
```

Then copy the syntax files:

```bash
cp -r editor/vim/* ~/.vim/
# Or for Neovim:
cp -r editor/vim/* ~/.config/nvim/
```

For LSP support in Neovim, add to your config (requires nvim-lspconfig):

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

configs.mdhavers = {
  default_config = {
    cmd = { 'mdhavers-lsp' },
    filetypes = { 'mdhavers' },
    root_dir = lspconfig.util.find_git_ancestor,
    single_file_support = true,
  },
}

lspconfig.mdhavers.setup({})
```

### TextMate/Sublime Text

Use the TextMate grammar file at `editor/mdhavers.tmLanguage.json`.

### Other Editors

Any editor with LSP support can use mdhavers-lsp. Configure it to:
- Run command: `mdhavers-lsp`
- File types: `*.braw`
- Communication: stdio

## License

MIT

---

*"This is havers, but it's working havers!"*

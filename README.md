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
| `crabbit(str)` | grumpy | Uppercase with "!": shout |
| `cannie(x)` | careful | Check if value is safe/valid |
| `glaikit(x, type)` | silly | Check if wrong type |
| `tattie_scone(s, n)` | potato scone | Repeat string with \| separator |
| `haggis_hunt(s, needle)` | haggis hunt | Find all occurrences of substring |
| `sporran_fill(s, w, c)` | sporran fill | Center-pad string |

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
| `log(n)` | Natural logarithm |
| `log10(n)` | Base 10 logarithm |
| `PI` | Pi constant (3.14159...) |
| `E` | Euler's number (2.71828...) |

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

# Check for errors
mdhavers check program.braw

# Format code (makes it look braw!)
mdhavers fmt program.braw
mdhavers fmt program.braw --check  # check only, dinnae modify

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
- `scottish_pub.braw` - A wee Scottish pub simulation (classes, dicts, HOF)
- `ceilidh.braw` - Scottish dance party (math functions, lists, shuffling)
- `modules_demo.braw` - Demonstrating the module import system
- `operator_overload.braw` - Operator overloading with classes
- `assert_demo.braw` - Assertions with mak_siccar
- `test_example.braw` - Testing library demonstration
- `scots_words.braw` - Scottish vocabulary functions demo
- `prelude_demo.braw` - Auto-loaded prelude functions demo
- `new_functions.braw` - New higher-order functions demo
- `spread.braw` - Spread operator (...) examples
- `pipe.braw` - Pipe operator (|>) examples
- `defaults.braw` - Default parameter values (staundart values)
- `destructure.braw` - Destructuring assignment examples
- `scots_fun.braw` - New Scots vocabulary functions demo
- `lib/maths.braw` - Mathematics utility library
- `lib/strings.braw` - String manipulation library
- `lib/collections.braw` - Data structures (stacks, queues, sets)
- `lib/functional.braw` - Functional programming utilities
- `lib/testing.braw` - Testing framework with assertions

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

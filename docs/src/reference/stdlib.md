# Standard Library Reference

mdhavers includes a standard library of modules in `examples/lib/`.

## Available Modules

| Module | Description |
|--------|-------------|
| `maths.braw` | Mathematical utilities |
| `strings.braw` | String manipulation |
| `collections.braw` | Data structures |
| `functional.braw` | Functional programming |
| `testing.braw` | Testing framework |
| `datetime.braw` | Date and time |
| `colors.braw` | Terminal colors |
| `logging.braw` | Logging utilities |
| `config.braw` | Configuration handling |
| `validate.braw` | Input validation |
| `patterns.braw` | Common patterns |

## Using Standard Library

```scots
# Import a standard library module
fetch "examples/lib/collections"
fetch "examples/lib/strings" tae str
```

## collections.braw

Data structures: Stack, Queue, Set.

### Stack

```scots
fetch "examples/lib/collections"

ken stack = Stack()
stack.push(1)
stack.push(2)
stack.push(3)

blether stack.pop()   # 3
blether stack.peek()  # 2
blether stack.size()  # 2
blether stack.is_empty()  # nae
```

### Queue

```scots
ken queue = Queue()
queue.enqueue("first")
queue.enqueue("second")
queue.enqueue("third")

blether queue.dequeue()  # "first"
blether queue.peek()     # "second"
blether queue.size()     # 2
```

### Set

```scots
ken set = Set()
set.add(1)
set.add(2)
set.add(1)  # Duplicate, ignored

blether set.size()      # 2
blether set.has(1)      # aye
blether set.to_list()   # [1, 2]
```

## strings.braw

String manipulation utilities.

```scots
fetch "examples/lib/strings" tae str

# Capitalization
blether str["capitalize"]("hello")      # "Hello"
blether str["title_case"]("hello world") # "Hello World"

# Case conversion
blether str["snake_to_camel"]("my_var")  # "myVar"
blether str["camel_to_snake"]("myVar")   # "my_var"

# String utilities
blether str["repeat"]("ab", 3)   # "ababab"
blether str["reverse"]("hello")  # "olleh"
blether str["is_blank"]("   ")   # aye
```

## maths.braw

Mathematical utilities beyond built-ins.

```scots
fetch "examples/lib/maths" tae m

# Basic operations
blether m["square"](5)        # 25
blether m["cube"](3)          # 27

# Geometry
blether m["circle_area"](5)       # ~78.54
blether m["rectangle_area"](4, 5) # 20
blether m["distance"](0, 0, 3, 4) # 5

# Statistics
blether m["mean"]([1,2,3,4,5])        # 3
blether m["standard_deviation"]([1,2,3,4,5])
```

## functional.braw

Functional programming utilities.

```scots
fetch "examples/lib/functional" tae f

# Composition
ken add_one = |x| x + 1
ken double = |x| x * 2
ken composed = f["compose"](double, add_one)
blether composed(5)  # 12 ((5+1) * 2)

# Partial application
ken add = |a, b| a + b
ken add_five = f["partial"](add, 5)
blether add_five(10)  # 15

# Pipeline
ken result = f["pipe"](5, [add_one, double, add_one])
blether result  # 13

# Utilities
blether f["identity"](42)     # 42
blether f["constant"](5)()    # 5
blether f["flip"](|a,b| a-b)(3, 10)  # 7
```

## testing.braw

Simple testing framework.

```scots
fetch "examples/lib/testing"

# Create a test suite
ken suite = TestSuite("Math Tests")

# Add tests
suite.test("addition", || {
    assert_equal(1 + 1, 2)
})

suite.test("multiplication", || {
    assert_equal(3 * 4, 12)
})

# Run tests
suite.run()
```

Output:
```
Running: Math Tests
  ✓ addition
  ✓ multiplication
Results: 2 passed, 0 failed
```

## datetime.braw

Date and time utilities.

```scots
fetch "examples/lib/datetime" tae dt

# Current time
ken now = dt["now"]()

# Formatting
blether dt["format_date"](now, "DD/MM/YYYY")
blether dt["format_time"](now, "HH:mm:ss")

# Calculations
ken tomorrow = dt["add_days"](now, 1)
ken next_week = dt["add_days"](now, 7)

# Comparisons
blether dt["is_before"](now, tomorrow)  # aye
blether dt["days_between"](now, next_week)  # 7
```

## colors.braw

Terminal colors and formatting.

```scots
fetch "examples/lib/colors" tae c

blether c["red"]("Error!")
blether c["green"]("Success!")
blether c["yellow"]("Warning")
blether c["blue"]("Info")
blether c["bold"]("Important")
blether c["underline"]("Link")

# Combine styles
blether c["bold"](c["red"]("Critical Error!"))
```

## logging.braw

Logging with levels and formatting.

```scots
fetch "examples/lib/logging"

ken logger = Logger("MyApp")

logger.debug("Debug message")
logger.info("Information")
logger.warn("Warning!")
logger.error("Error occurred")

# Set log level
logger.set_level("warn")  # Only warn and error shown
```

Output:
```
[DEBUG] MyApp: Debug message
[INFO] MyApp: Information
[WARN] MyApp: Warning!
[ERROR] MyApp: Error occurred
```

## config.braw

Configuration file handling.

```scots
fetch "examples/lib/config"

# Create config
ken config = Config()
config.set("database.host", "localhost")
config.set("database.port", 5432)
config.set("app.debug", aye)

# Get values
blether config.get("database.host")      # "localhost"
blether config.get("missing", "default") # "default"

# Check existence
blether config.has("database.port")  # aye
```

## validate.braw

Input validation utilities.

```scots
fetch "examples/lib/validate" tae v

# String validation
blether v["is_email"]("user@example.com")  # aye
blether v["is_url"]("https://example.com") # aye
blether v["is_numeric"]("12345")           # aye

# Number validation
blether v["in_range"](5, 1, 10)  # aye
blether v["is_positive"](5)     # aye

# List validation
blether v["not_empty"]([1, 2])  # aye
blether v["all_match"]([2,4,6], |x| x % 2 == 0)  # aye
```

## patterns.braw

Common programming patterns.

```scots
fetch "examples/lib/patterns"

# Singleton
ken instance = Singleton.get_instance()

# Builder pattern
ken person = PersonBuilder()
    .name("Hamish")
    .age(30)
    .city("Glasgow")
    .build()

# Observer pattern
ken subject = Subject()
subject.subscribe(|data| blether f"Got: {data}")
subject.notify("Hello!")
```

## Creating Your Own Modules

Create a `.braw` file with functions and classes:

```scots
# my_utils.braw

ken VERSION = "1.0.0"

dae greet(name) {
    gie f"Hello, {name}!"
}

dae calculate(a, b, op) {
    keek op {
        whan "add" -> { gie a + b }
        whan "sub" -> { gie a - b }
        whan _ -> { gie naething }
    }
}

kin Helper {
    dae init() {
        masel.count = 0
    }

    dae increment() {
        masel.count = masel.count + 1
        gie masel.count
    }
}
```

Use your module:

```scots
fetch "my_utils"

blether VERSION           # "1.0.0"
blether greet("World")    # "Hello, World!"
blether calculate(5, 3, "add")  # 8

ken helper = Helper()
blether helper.increment()  # 1
blether helper.increment()  # 2
```

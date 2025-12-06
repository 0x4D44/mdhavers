# Modules

Organize your code into separate files with the `fetch` keyword.

## Basic Module Import

Use `fetch` to import another `.braw` file:

```scots
# In main.braw
fetch "helpers"

# Now you can use functions from helpers.braw
helper_function()
```

The `.braw` extension is optional:

```scots
fetch "helpers"      # Looks for helpers.braw
fetch "helpers.braw" # Same thing
```

## Module Resolution

Modules are resolved relative to the current file:

```
project/
├── main.braw
├── utils.braw
└── lib/
    ├── math.braw
    └── strings.braw
```

```scots
# In main.braw
fetch "utils"          # ./utils.braw
fetch "lib/math"       # ./lib/math.braw
fetch "lib/strings"    # ./lib/strings.braw
```

## Importing with Aliases

Use `tae` to import a module with a namespace:

```scots
fetch "lib/math" tae m

# Access with namespace
ken result = m["square"](5)
ken pi_value = m["PI"]
```

This prevents name collisions when multiple modules define the same function names.

## Creating a Module

Any `.braw` file can be a module. Variables and functions defined at the top level are exported:

```scots
# math_utils.braw

ken PI = 3.14159
ken E = 2.71828

dae square(x) {
    gie x * x
}

dae cube(x) {
    gie x * x * x
}

dae circle_area(radius) {
    gie PI * radius * radius
}
```

```scots
# main.braw
fetch "math_utils"

blether square(5)  # 25
blether PI         # 3.14159
```

## Module with Classes

```scots
# shapes.braw

kin Rectangle {
    dae init(width, height) {
        masel.width = width
        masel.height = height
    }

    dae area() {
        gie masel.width * masel.height
    }
}

kin Circle {
    dae init(radius) {
        masel.radius = radius
    }

    dae area() {
        gie 3.14159 * masel.radius * masel.radius
    }
}
```

```scots
# main.braw
fetch "shapes"

ken rect = Rectangle(4, 5)
blether rect.area()  # 20

ken circle = Circle(3)
blether circle.area()  # 28.27...
```

## Nested Modules

Modules can import other modules:

```scots
# lib/validation.braw
dae is_valid_email(email) {
    gie contains(email, "@")
}
```

```scots
# lib/user.braw
fetch "lib/validation"  # Note: relative to project root

kin User {
    dae init(name, email) {
        gin nae is_valid_email(email) {
            blether "Warning: invalid email!"
        }
        masel.name = name
        masel.email = email
    }
}
```

```scots
# main.braw
fetch "lib/user"

ken user = User("Hamish", "hamish@example.com")
```

## Project Organization

Here's a recommended structure for larger projects:

```
myproject/
├── main.braw           # Entry point
├── config.braw         # Configuration
├── lib/
│   ├── core.braw       # Core utilities
│   ├── data.braw       # Data structures
│   └── utils.braw      # Helper functions
├── models/
│   ├── user.braw
│   └── product.braw
└── tests/
    └── test_core.braw
```

```scots
# main.braw
fetch "config"
fetch "lib/core"
fetch "lib/utils"
fetch "models/user"

# Your main program here
```

## Example: Utility Library

```scots
# lib/strings.braw - String manipulation utilities

dae capitalize(text) {
    gin len(text) == 0 {
        gie ""
    }
    ken first = upper(text[0])
    ken rest = gin len(text) > 1 than scran(text, 1, len(text)) ither ""
    gie first + rest
}

dae title_case(text) {
    ken words = split(text, " ")
    ken titled = gaun(words, |w| capitalize(lower(w)))
    gie join(titled, " ")
}

dae snake_to_camel(text) {
    ken parts = split(text, "_")
    ken first = lower(parts[0])
    ken rest = gaun(tail(parts), |p| capitalize(lower(p)))
    gie first + join(rest, "")
}

dae repeat(text, times) {
    ken result = ""
    fer i in 0..times {
        result = result + text
    }
    gie result
}
```

```scots
# main.braw
fetch "lib/strings" tae str

blether str["capitalize"]("hello")       # "Hello"
blether str["title_case"]("hello world") # "Hello World"
blether str["snake_to_camel"]("my_var")  # "myVar"
blether str["repeat"]("ab", 3)           # "ababab"
```

## Example: Data Models

```scots
# models/person.braw

kin Person {
    dae init(name, age) {
        masel.name = name
        masel.age = age
    }

    dae is_adult() {
        gie masel.age >= 18
    }

    dae to_dict() {
        gie {
            "name": masel.name,
            "age": masel.age
        }
    }
}

dae create_person(data) {
    gie Person(data["name"], data["age"])
}
```

```scots
# main.braw
fetch "models/person"

ken hamish = Person("Hamish", 30)
blether hamish.is_adult()  # aye

ken data = {"name": "Morag", "age": 25}
ken morag = create_person(data)
blether morag.to_dict()
```

## Standard Library Modules

mdhavers comes with several useful library modules in `examples/lib/`:

| Module | Description |
|--------|-------------|
| `maths.braw` | Mathematical utilities |
| `strings.braw` | String manipulation |
| `collections.braw` | Data structures (Stack, Queue, Set) |
| `functional.braw` | Functional programming helpers |
| `testing.braw` | Testing framework |
| `datetime.braw` | Date and time utilities |
| `colors.braw` | Terminal colors |
| `logging.braw` | Logging utilities |

Example usage:

```scots
fetch "examples/lib/collections"

ken stack = Stack()
stack.push(1)
stack.push(2)
blether stack.pop()  # 2
```

## Best Practices

### 1. One Concept Per Module

```scots
# Good: Focused module
# validation.braw
dae is_email(s) { ... }
dae is_url(s) { ... }
dae is_phone(s) { ... }

# Avoid: Kitchen sink module
# utils.braw with unrelated functions
```

### 2. Clear Naming

```scots
# Good: Descriptive names
fetch "lib/user_validation"
fetch "lib/string_helpers"

# Avoid: Vague names
fetch "lib/stuff"
fetch "lib/misc"
```

### 3. Document Your Modules

```scots
# date_utils.braw
# Date manipulation utilities for mdhavers
#
# Functions:
#   format_date(timestamp) - Format a timestamp as "DD/MM/YYYY"
#   days_between(date1, date2) - Calculate days between dates
#   is_weekend(timestamp) - Check if date is a weekend

dae format_date(timestamp) {
    # Implementation
}
```

### 4. Avoid Circular Imports

Don't have module A import module B while B imports A:

```scots
# BAD: Circular dependency
# a.braw
fetch "b"

# b.braw
fetch "a"  # This will cause problems!
```

Instead, extract shared code into a third module:

```scots
# Good: Shared module
# shared.braw - common functions

# a.braw
fetch "shared"

# b.braw
fetch "shared"
```

## Exercises

1. **String Utils Module**: Create a module with string transformation functions

2. **Math Module**: Create a module with mathematical utilities (factorial, fibonacci, etc.)

3. **Multi-file Project**: Create a simple project with multiple modules

<details>
<summary>Solutions</summary>

```scots
# 1. String Utils Module (string_utils.braw)
dae reverse_string(s) {
    ken chars = []
    fer c in s {
        shove(chars, c)
    }
    gie join(reverse(chars), "")
}

dae word_count(s) {
    ken words = sieve(split(s, " "), |w| len(w) > 0)
    gie len(words)
}

dae truncate(s, max_len, suffix = "...") {
    gin len(s) <= max_len {
        gie s
    }
    gie scran(s, 0, max_len - len(suffix)) + suffix
}

# 2. Math Module (math_utils.braw)
dae factorial(n) {
    gin n <= 1 {
        gie 1
    }
    gie n * factorial(n - 1)
}

dae fibonacci(n) {
    gin n <= 1 {
        gie n
    }
    ken a = 0
    ken b = 1
    fer i in 2..(n + 1) {
        ken temp = a + b
        a = b
        b = temp
    }
    gie b
}

dae is_prime(n) {
    gin n < 2 {
        gie nae
    }
    fer i in 2..n {
        gin n % i == 0 {
            gie nae
        }
    }
    gie aye
}

# 3. Multi-file Project
# --- project/config.braw ---
ken APP_NAME = "My App"
ken VERSION = "1.0.0"
ken DEBUG = aye

# --- project/lib/logger.braw ---
fetch "config"

dae log(message) {
    gin DEBUG {
        blether f"[{APP_NAME}] {message}"
    }
}

# --- project/main.braw ---
fetch "config"
fetch "lib/logger"

log(f"Starting {APP_NAME} v{VERSION}")
# Do main program work
log("Done!")
```

</details>

## Next Steps

Learn about [error handling](./08-error-handling.md) to make your programs more robust.

# Basics

Learn the fundamentals of mdhavers: variables, types, operators, and expressions.

## Variables

Use `ken` (Scots for "to know") to declare variables:

```scots
ken name = "Hamish"
ken age = 42
ken pi = 3.14159
ken is_scottish = aye
```

Variables can be reassigned:

```scots
ken count = 1
count = 2
count = count + 1
blether count  # 3
```

### Naming Rules

- Start with a letter or underscore
- Can contain letters, numbers, and underscores
- Case-sensitive (`name` and `Name` are different)

```scots
ken valid_name = 1
ken _private = 2
ken camelCase = 3
ken snake_case = 4

# These would be errors:
# ken 2things = 5   # Can't start with number
# ken my-var = 6    # No hyphens
```

## Data Types

### Numbers

**Integers** are whole numbers:
```scots
ken positive = 42
ken negative = -17
ken zero = 0
```

**Floats** are decimal numbers:
```scots
ken pi = 3.14159
ken temperature = -5.5
ken tiny = 0.001
```

Arithmetic works as expected:
```scots
blether 10 + 3    # 13
blether 10 - 3    # 7
blether 10 * 3    # 30
blether 10 / 3    # 3.333...
blether 10 % 3    # 1 (remainder)
```

### Strings

Text enclosed in quotes:

```scots
ken single = 'Hello'
ken double = "World"
ken with_quotes = "She said \"Braw!\""
```

**Concatenation** with `+`:
```scots
ken greeting = "Hullo, " + "World!"
blether greeting  # "Hullo, World!"
```

**F-strings** for interpolation:
```scots
ken name = "Hamish"
ken age = 42
blether f"I'm {name}, {age} years auld"
# "I'm Hamish, 42 years auld"

# Expressions work too
blether f"2 + 2 = {2 + 2}"  # "2 + 2 = 4"
```

### Booleans

True and false, Scottish style:

```scots
ken yes = aye      # true
ken no = nae       # false
```

### Null

The absence of a value:

```scots
ken nothing = naething
```

### Lists

Ordered collections:

```scots
ken numbers = [1, 2, 3, 4, 5]
ken names = ["Hamish", "Morag", "Angus"]
ken mixed = [1, "two", 3.0, aye]
ken empty = []
```

**Access by index** (0-based):
```scots
ken cities = ["Edinburgh", "Glasgow", "Aberdeen"]
blether cities[0]    # "Edinburgh"
blether cities[2]    # "Aberdeen"
blether cities[-1]   # "Aberdeen" (negative = from end)
```

**Modify elements:**
```scots
ken scores = [85, 90, 78]
scores[1] = 95
blether scores  # [85, 95, 78]
```

### Dictionaries

Key-value pairs:

```scots
ken person = {
    "name": "Rabbie Burns",
    "job": "Poet",
    "born": 1759
}

blether person["name"]  # "Rabbie Burns"
person["died"] = 1796   # Add new key
```

Keys can be strings or numbers:
```scots
ken lookup = {
    1: "one",
    2: "two",
    "three": 3
}
```

## Operators

### Arithmetic

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `5 + 3` → `8` |
| `-` | Subtraction | `5 - 3` → `2` |
| `*` | Multiplication | `5 * 3` → `15` |
| `/` | Division | `5 / 3` → `1.666...` |
| `%` | Modulo | `5 % 3` → `2` |

### Comparison

| Operator | Description | Example |
|----------|-------------|---------|
| `==` | Equal | `5 == 5` → `aye` |
| `!=` | Not equal | `5 != 3` → `aye` |
| `<` | Less than | `3 < 5` → `aye` |
| `>` | Greater than | `5 > 3` → `aye` |
| `<=` | Less or equal | `5 <= 5` → `aye` |
| `>=` | Greater or equal | `5 >= 3` → `aye` |

### Logical

| Operator | Description | Example |
|----------|-------------|---------|
| `an` | And | `aye an aye` → `aye` |
| `or` | Or | `aye or nae` → `aye` |
| `nae` | Not | `nae aye` → `nae` |

```scots
ken age = 25
ken has_id = aye

gin age >= 18 an has_id {
    blether "Ye can buy whisky!"
}
```

### Compound Assignment

Shortcuts for updating variables:

```scots
ken x = 10
x += 5    # x = x + 5 → 15
x -= 3    # x = x - 3 → 12
x *= 2    # x = x * 2 → 24
x /= 4    # x = x / 4 → 6
```

## Type Checking and Conversion

### Check Type

```scots
ken x = 42
blether whit_kind(x)       # "integer"
blether whit_kind("hello") # "string"
blether whit_kind([1,2,3]) # "list"
```

### Type Checking

```scots
ken x = 42
blether is_a(x, "integer")  # aye
blether is_a(x, "string")   # nae
```

### Convert Types

```scots
# To string
ken num = 42
ken text = tae_string(num)  # "42"

# To integer
ken s = "123"
ken n = tae_int(s)  # 123

# To float
ken f = tae_float("3.14")  # 3.14

# To boolean
ken b = tae_bool(1)  # aye
ken b2 = tae_bool(0) # nae
```

## Comments

```scots
# This is a single-line comment

ken x = 5  # Comment at end of line

# Multiple
# lines
# of comments
```

## Input and Output

### Output with blether

```scots
blether "Hello, World!"
blether 42
blether [1, 2, 3]
```

### Input with speir

```scots
ken name = speir "Whit's yer name? "
ken age = tae_int(speir "How auld are ye? ")
blether f"Hullo, {name}! Ye're {age} years auld."
```

## Exercises

Try these to practice:

1. **Variable swap**: Create two variables and swap their values
   ```scots
   ken a = 1
   ken b = 2
   # Swap them so a = 2 and b = 1
   ```

2. **Temperature converter**: Convert Celsius to Fahrenheit
   ```scots
   ken celsius = 20
   # Formula: F = C * 9/5 + 32
   ```

3. **List statistics**: Given a list of numbers, calculate the sum
   ```scots
   ken numbers = [10, 20, 30, 40, 50]
   # Calculate the total
   ```

<details>
<summary>Solutions</summary>

```scots
# 1. Variable swap
ken a = 1
ken b = 2
ken temp = a
a = b
b = temp
blether f"a = {a}, b = {b}"  # a = 2, b = 1

# 2. Temperature converter
ken celsius = 20
ken fahrenheit = celsius * 9 / 5 + 32
blether f"{celsius}C = {fahrenheit}F"  # 20C = 68F

# 3. List statistics
ken numbers = [10, 20, 30, 40, 50]
ken total = sumaw(numbers)
blether f"Sum: {total}"  # Sum: 150
```

</details>

## Next Steps

Now that you understand the basics, learn about [control flow](./02-control-flow.md) - how to make decisions and repeat actions in your programs.

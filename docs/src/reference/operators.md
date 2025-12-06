# Operators Reference

Complete reference of all operators in mdhavers.

## Arithmetic Operators

| Operator | Description | Example | Result |
|----------|-------------|---------|--------|
| `+` | Addition | `5 + 3` | `8` |
| `-` | Subtraction | `5 - 3` | `2` |
| `*` | Multiplication | `5 * 3` | `15` |
| `/` | Division | `10 / 3` | `3.333...` |
| `%` | Modulo (remainder) | `10 % 3` | `1` |

### Integer vs Float Division

Division always returns a float:
```scots
blether 10 / 3   # 3.333...
blether 10 / 2   # 5.0
```

For integer division, use floor:
```scots
blether floor(10 / 3)  # 3
```

## Comparison Operators

| Operator | Description | Example | Result |
|----------|-------------|---------|--------|
| `==` | Equal | `5 == 5` | `aye` |
| `!=` | Not equal | `5 != 3` | `aye` |
| `<` | Less than | `3 < 5` | `aye` |
| `>` | Greater than | `5 > 3` | `aye` |
| `<=` | Less than or equal | `5 <= 5` | `aye` |
| `>=` | Greater than or equal | `5 >= 3` | `aye` |

### Comparing Different Types

```scots
blether 5 == 5.0    # aye (numbers compare by value)
blether "a" < "b"   # aye (strings compare lexically)
blether [1,2] == [1,2]  # aye (lists compare by content)
```

## Logical Operators

| Operator | Description | Example | Result |
|----------|-------------|---------|--------|
| `an` | Logical AND | `aye an aye` | `aye` |
| `or` | Logical OR | `aye or nae` | `aye` |
| `nae` | Logical NOT | `nae aye` | `nae` |

### Truth Tables

**AND (`an`)**
| A | B | A an B |
|---|---|--------|
| aye | aye | aye |
| aye | nae | nae |
| nae | aye | nae |
| nae | nae | nae |

**OR (`or`)**
| A | B | A or B |
|---|---|--------|
| aye | aye | aye |
| aye | nae | aye |
| nae | aye | aye |
| nae | nae | nae |

**NOT (`nae`)**
| A | nae A |
|---|-------|
| aye | nae |
| nae | aye |

### Short-Circuit Evaluation

```scots
# Second condition not evaluated if first is false
gin nae an expensive_check() {
    # expensive_check() never called
}

# Second condition not evaluated if first is true
gin aye or expensive_check() {
    # expensive_check() never called
}
```

## Assignment Operators

| Operator | Description | Equivalent |
|----------|-------------|------------|
| `=` | Assignment | `x = 5` |
| `+=` | Add and assign | `x = x + 5` |
| `-=` | Subtract and assign | `x = x - 5` |
| `*=` | Multiply and assign | `x = x * 5` |
| `/=` | Divide and assign | `x = x / 5` |

```scots
ken x = 10
x += 5   # x is now 15
x -= 3   # x is now 12
x *= 2   # x is now 24
x /= 4   # x is now 6
```

## String Operators

| Operator | Description | Example | Result |
|----------|-------------|---------|--------|
| `+` | Concatenation | `"Hello" + " World"` | `"Hello World"` |
| `*` | Repetition | `"ab" * 3` | `"ababab"` |

```scots
ken greeting = "Hello"
ken name = "World"
blether greeting + ", " + name + "!"  # "Hello, World!"
blether "=" * 20  # "===================="
```

## List Operators

| Operator | Description | Example | Result |
|----------|-------------|---------|--------|
| `+` | Concatenation | `[1,2] + [3,4]` | `[1,2,3,4]` |
| `*` | Repetition | `[1,2] * 2` | `[1,2,1,2]` |

```scots
ken a = [1, 2, 3]
ken b = [4, 5, 6]
blether a + b      # [1, 2, 3, 4, 5, 6]
blether [0] * 5    # [0, 0, 0, 0, 0]
```

## Range Operator

| Operator | Description | Example | Result |
|----------|-------------|---------|--------|
| `..` | Range (exclusive end) | `1..5` | `[1, 2, 3, 4]` |

```scots
fer i in 1..5 {
    blether i  # Prints 1, 2, 3, 4
}

ken nums = 0..3
blether nums  # [0, 1, 2]
```

## Spread Operator

| Operator | Description |
|----------|-------------|
| `...` | Spread/expand elements |

### In Lists

```scots
ken a = [1, 2, 3]
ken b = [4, 5, 6]
ken combined = [...a, ...b]  # [1, 2, 3, 4, 5, 6]
ken with_extra = [0, ...a, 99]  # [0, 1, 2, 3, 99]
```

### In Function Calls

```scots
dae sum_three(x, y, z) {
    gie x + y + z
}

ken args = [10, 20, 30]
blether sum_three(...args)  # 60
```

### Spread Strings

```scots
ken letters = [..."hello"]  # ["h", "e", "l", "l", "o"]
```

## Pipe Operator

| Operator | Description |
|----------|-------------|
| `\|>` | Pipe (chain function calls) |

```scots
ken result = 5 |> |x| x * 2 |> |x| x + 1
blether result  # 11

dae double(x) { gie x * 2 }
dae add_one(x) { gie x + 1 }

blether 5 |> double |> add_one  # 11
```

## Index and Access Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `[]` | Index access | `list[0]`, `dict["key"]` |
| `.` | Property access | `object.property` |

```scots
ken list = [10, 20, 30]
blether list[0]   # 10
blether list[-1]  # 30 (last element)

ken dict = {"name": "Hamish", "age": 30}
blether dict["name"]  # "Hamish"

ken person = Person("Hamish")
blether person.name  # "Hamish"
```

## Arrow Operator

| Operator | Description |
|----------|-------------|
| `->` | Pattern match result |

```scots
keek value {
    whan 1 -> { blether "one" }
    whan 2 -> { blether "two" }
    whan _ -> { blether "other" }
}
```

## Lambda Operator

| Operator | Description |
|----------|-------------|
| `\|...\|` | Lambda/anonymous function |

```scots
ken square = |x| x * x
ken add = |a, b| a + b
ken greet = || "Hello!"
```

## Operator Precedence

From highest to lowest:

1. `()` - Parentheses
2. `.` `[]` - Member access
3. `|x|` - Lambda
4. `nae` - Unary not
5. `*` `/` `%` - Multiplication, division, modulo
6. `+` `-` - Addition, subtraction
7. `..` - Range
8. `<` `>` `<=` `>=` - Comparison
9. `==` `!=` - Equality
10. `an` - Logical AND
11. `or` - Logical OR
12. `|>` - Pipe
13. `=` `+=` `-=` `*=` `/=` - Assignment

### Using Parentheses

```scots
blether 2 + 3 * 4     # 14 (multiplication first)
blether (2 + 3) * 4   # 20 (parentheses override)

blether aye or nae an nae  # aye (an has higher precedence)
blether (aye or nae) an nae  # nae
```

## Operator Overloading

Custom classes can define how operators work:

| Method | Operator |
|--------|----------|
| `__pit_thegither__` | `+` |
| `__tak_awa__` | `-` |
| `__times__` | `*` |
| `__pairt__` | `/` |
| `__lave__` | `%` |
| `__same_as__` | `==` |
| `__differs_fae__` | `!=` |
| `__wee_er__` | `<` |
| `__wee_er_or_same__` | `<=` |
| `__muckle_er__` | `>` |
| `__muckle_er_or_same__` | `>=` |

See [Operator Overloading](../advanced/operator-overloading.md) for details.

# Functional Programming

Master higher-order functions and functional patterns in mdhavers.

## Higher-Order Functions

Higher-order functions take functions as arguments or return functions.

### gaun (map)

Transform each element of a list:

```scots
ken numbers = [1, 2, 3, 4, 5]

# Double each number
ken doubled = gaun(numbers, |x| x * 2)
blether doubled  # [2, 4, 6, 8, 10]

# Square each number
ken squared = gaun(numbers, |x| x * x)
blether squared  # [1, 4, 9, 16, 25]

# Convert to strings
ken strings = gaun(numbers, |x| tae_string(x))
blether strings  # ["1", "2", "3", "4", "5"]
```

### sieve (filter)

Keep only elements that match a condition:

```scots
ken numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

# Keep even numbers
ken evens = sieve(numbers, |x| x % 2 == 0)
blether evens  # [2, 4, 6, 8, 10]

# Keep numbers > 5
ken big = sieve(numbers, |x| x > 5)
blether big  # [6, 7, 8, 9, 10]

# Keep strings longer than 3 characters
ken words = ["a", "bee", "cat", "door", "elephant"]
ken long_words = sieve(words, |w| len(w) > 3)
blether long_words  # ["door", "elephant"]
```

### tumble (reduce/fold)

Combine all elements into a single value:

```scots
ken numbers = [1, 2, 3, 4, 5]

# Sum all numbers
ken sum = tumble(numbers, 0, |acc, x| acc + x)
blether sum  # 15

# Multiply all numbers (product)
ken product = tumble(numbers, 1, |acc, x| acc * x)
blether product  # 120

# Find maximum
ken maximum = tumble(numbers, numbers[0], |acc, x| gin x > acc than x ither acc)
blether maximum  # 5

# Join strings
ken words = ["Hello", "World", "!"]
ken sentence = tumble(words, "", |acc, w| acc + w + " ")
blether wheesht(sentence)  # "Hello World !"
```

### ilk (for-each)

Execute a function for each element (side effects):

```scots
ken items = ["haggis", "neeps", "tatties"]

dae print_item(item) {
    blether f"  - {item}"
}

blether "Scottish supper:"
ilk(items, print_item)
```

Output:
```
Scottish supper:
  - haggis
  - neeps
  - tatties
```

## The Pipe Operator

Chain operations fluently with `|>`:

```scots
ken result = 5 |> |x| x * 2 |> |x| x + 1
blether result  # 11

# With named functions
dae add_one(x) { gie x + 1 }
dae triple(x) { gie x * 3 }

ken chained = 5 |> add_one |> triple |> add_one
blether chained  # 19 (5 → 6 → 18 → 19)
```

### Data Processing Pipelines

```scots
ken numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

dae get_evens(list) {
    gie sieve(list, |x| x % 2 == 0)
}

dae double_all(list) {
    gie gaun(list, |x| x * 2)
}

# Pipeline: filter evens → double → sum
ken result = numbers |> get_evens |> double_all |> sumaw
blether result  # 60 (2+4+6+8+10 doubled = 4+8+12+16+20)
```

### String Processing Pipeline

```scots
dae trim_it(s) { gie wheesht(s) }
dae make_upper(s) { gie upper(s) }
dae add_greeting(s) { gie f"Hello, {s}!" }

ken greeting = "  hamish  " |> trim_it |> make_upper |> add_greeting
blether greeting  # "Hello, HAMISH!"
```

## Lambdas (Anonymous Functions)

```scots
# Single parameter
ken square = |x| x * x

# Multiple parameters
ken add = |a, b| a + b

# No parameters
ken get_pi = || 3.14159

# Multi-line lambdas
ken complex_op = |x| {
    ken doubled = x * 2
    ken tripled = x * 3
    gie doubled + tripled
}
```

## Closures

Functions that capture their environment:

```scots
dae make_adder(n) {
    gie |x| x + n
}

ken add_five = make_adder(5)
ken add_ten = make_adder(10)

blether add_five(3)   # 8
blether add_ten(3)    # 13
```

### Counter Example

```scots
dae make_counter() {
    ken count = 0
    gie || {
        count = count + 1
        gie count
    }
}

ken counter = make_counter()
blether counter()  # 1
blether counter()  # 2
blether counter()  # 3
```

## Function Composition

```scots
dae compose(f, g) {
    gie |x| f(g(x))
}

dae double(x) { gie x * 2 }
dae increment(x) { gie x + 1 }

ken double_then_inc = compose(increment, double)
ken inc_then_double = compose(double, increment)

blether double_then_inc(5)  # 11 (5 * 2 + 1)
blether inc_then_double(5)  # 12 ((5 + 1) * 2)
```

## Currying

Transform a multi-argument function into a chain of single-argument functions:

```scots
dae curry_add(a) {
    gie |b| a + b
}

ken add_five = curry_add(5)
blether add_five(10)  # 15

# Curry multiply
dae curry_multiply(a) {
    gie |b| a * b
}

ken double = curry_multiply(2)
ken triple = curry_multiply(3)

blether double(7)  # 14
blether triple(7)  # 21
```

## Practical Examples

### Data Transformation

```scots
ken people = [
    {"name": "Hamish", "age": 30, "city": "Glasgow"},
    {"name": "Morag", "age": 25, "city": "Edinburgh"},
    {"name": "Angus", "age": 45, "city": "Glasgow"},
    {"name": "Flora", "age": 35, "city": "Edinburgh"}
]

# Get names of people over 30 from Glasgow
ken result = people
ken glaswegians = sieve(result, |p| p["city"] == "Glasgow")
ken over_30 = sieve(glaswegians, |p| p["age"] >= 30)
ken names = gaun(over_30, |p| p["name"])
blether names  # ["Hamish", "Angus"]
```

### Statistical Functions

```scots
dae mean(numbers) {
    gin len(numbers) == 0 { gie 0 }
    gie tumble(numbers, 0, |acc, x| acc + x) / len(numbers)
}

dae variance(numbers) {
    ken avg = mean(numbers)
    ken squared_diffs = gaun(numbers, |x| (x - avg) * (x - avg))
    gie mean(squared_diffs)
}

dae std_dev(numbers) {
    gie sqrt(variance(numbers))
}

ken data = [2, 4, 4, 4, 5, 5, 7, 9]
blether f"Mean: {mean(data)}"
blether f"Variance: {variance(data)}"
blether f"Std Dev: {std_dev(data)}"
```

### List Utilities

```scots
# Take while condition is true
dae take_while(list, pred) {
    ken result = []
    fer item in list {
        gin nae pred(item) {
            brak
        }
        shove(result, item)
    }
    gie result
}

# Drop while condition is true
dae drop_while(list, pred) {
    ken dropping = aye
    ken result = []
    fer item in list {
        gin dropping an pred(item) {
            haud
        }
        dropping = nae
        shove(result, item)
    }
    gie result
}

ken nums = [1, 2, 3, 4, 5, 1, 2, 3]
blether take_while(nums, |x| x < 4)  # [1, 2, 3]
blether drop_while(nums, |x| x < 4)  # [4, 5, 1, 2, 3]
```

### Partial Application

```scots
dae partial(func, first_arg) {
    gie |second_arg| func(first_arg, second_arg)
}

dae multiply(a, b) {
    gie a * b
}

ken double = partial(multiply, 2)
ken triple = partial(multiply, 3)

ken numbers = [1, 2, 3, 4, 5]
blether gaun(numbers, double)  # [2, 4, 6, 8, 10]
blether gaun(numbers, triple)  # [3, 6, 9, 12, 15]
```

### Memoization

```scots
dae memoize(func) {
    ken cache = {}

    gie |arg| {
        ken key = tae_string(arg)
        gin dict_has(cache, key) {
            gie cache[key]
        }
        ken result = func(arg)
        cache[key] = result
        gie result
    }
}

# Slow fibonacci
dae slow_fib(n) {
    gin n <= 1 {
        gie n
    }
    gie slow_fib(n - 1) + slow_fib(n - 2)
}

# This is faster with memoization
ken fib = memoize(|n| {
    gin n <= 1 {
        gie n
    }
    gie fib(n - 1) + fib(n - 2)
})

blether fib(30)  # Much faster!
```

## Combining Multiple Operations

```scots
ken transactions = [
    {"type": "credit", "amount": 100},
    {"type": "debit", "amount": 50},
    {"type": "credit", "amount": 200},
    {"type": "debit", "amount": 75},
    {"type": "credit", "amount": 150}
]

# Calculate total credits
ken total_credits = tumble(
    sieve(transactions, |t| t["type"] == "credit"),
    0,
    |acc, t| acc + t["amount"]
)
blether f"Total credits: {total_credits}"  # 450

# Calculate balance
ken balance = tumble(transactions, 0, |acc, t| {
    gin t["type"] == "credit" {
        gie acc + t["amount"]
    } ither {
        gie acc - t["amount"]
    }
})
blether f"Balance: {balance}"  # 325
```

## Exercises

1. **Implement zip**: Create a function that pairs elements from two lists

2. **Group By**: Create a function that groups items by a key function

3. **Flatten**: Create a function that flattens nested lists

<details>
<summary>Solutions</summary>

```scots
# 1. Implement zip
dae zip(list1, list2) {
    ken result = []
    ken length = min(len(list1), len(list2))
    fer i in 0..length {
        shove(result, [list1[i], list2[i]])
    }
    gie result
}

ken names = ["Alice", "Bob", "Charlie"]
ken scores = [85, 92, 78]
blether zip(names, scores)
# [["Alice", 85], ["Bob", 92], ["Charlie", 78]]

# 2. Group By
dae group_by(list, key_func) {
    ken groups = {}
    fer item in list {
        ken key = key_func(item)
        gin nae dict_has(groups, key) {
            groups[key] = []
        }
        shove(groups[key], item)
    }
    gie groups
}

ken people = [
    {"name": "Alice", "age": 30},
    {"name": "Bob", "age": 25},
    {"name": "Charlie", "age": 30},
    {"name": "Diana", "age": 25}
]

ken by_age = group_by(people, |p| tae_string(p["age"]))
blether by_age

# 3. Flatten (one level)
dae flatten(nested) {
    ken result = []
    fer item in nested {
        gin is_a(item, "list") {
            fer sub in item {
                shove(result, sub)
            }
        } ither {
            shove(result, item)
        }
    }
    gie result
}

# Deep flatten (recursive)
dae deep_flatten(nested) {
    ken result = []
    fer item in nested {
        gin is_a(item, "list") {
            ken flattened = deep_flatten(item)
            fer sub in flattened {
                shove(result, sub)
            }
        } ither {
            shove(result, item)
        }
    }
    gie result
}

blether flatten([[1, 2], [3, 4], [5]])  # [1, 2, 3, 4, 5]
blether deep_flatten([[1, [2, 3]], [[4, 5], 6]])  # [1, 2, 3, 4, 5, 6]
```

</details>

## Next Steps

You've completed the Learning section! Now explore:

- [Operator Overloading](../advanced/operator-overloading.md) for custom types
- [Destructuring](../advanced/destructuring.md) for elegant unpacking
- [Reference Documentation](../reference/keywords.md) for complete language details

# Functions

Organize your code into reusable, named blocks with `dae`.

## Defining Functions

Use `dae` (Scots for "to do") to define functions:

```scots
dae greet() {
    blether "Hullo there!"
}

# Call the function
greet()  # "Hullo there!"
```

## Parameters

Functions can accept parameters:

```scots
dae greet(name) {
    blether f"Hullo, {name}!"
}

greet("Hamish")  # "Hullo, Hamish!"
greet("Morag")   # "Hullo, Morag!"
```

Multiple parameters:

```scots
dae introduce(name, age, city) {
    blether f"{name} is {age} years auld, fae {city}."
}

introduce("Angus", 35, "Glasgow")
# "Angus is 35 years auld, fae Glasgow."
```

## Return Values with gie

Use `gie` (Scots for "to give") to return values:

```scots
dae add(a, b) {
    gie a + b
}

ken result = add(5, 3)
blether result  # 8
```

Return early:

```scots
dae is_even(n) {
    gin n % 2 == 0 {
        gie aye
    }
    gie nae
}

blether is_even(4)  # aye
blether is_even(7)  # nae
```

Functions without explicit `gie` return `naething`:

```scots
dae say_hello() {
    blether "Hello!"
}

ken result = say_hello()
blether result  # naething
```

## Default Parameters

Provide default values for parameters:

```scots
dae greet(name, greeting = "Hullo") {
    blether f"{greeting}, {name}!"
}

greet("Hamish")              # "Hullo, Hamish!"
greet("Morag", "Och aye")    # "Och aye, Morag!"
```

Multiple defaults:

```scots
dae make_order(item, quantity = 1, price = 10) {
    gie quantity * price
}

blether make_order("haggis")           # 10 (1 * 10)
blether make_order("whisky", 3)        # 30 (3 * 10)
blether make_order("shortbread", 5, 2) # 10 (5 * 2)
```

**Note:** Parameters with defaults must come after parameters without defaults.

## Recursion

Functions can call themselves:

```scots
dae factorial(n) {
    gin n <= 1 {
        gie 1
    }
    gie n * factorial(n - 1)
}

blether factorial(5)  # 120 (5 * 4 * 3 * 2 * 1)
```

Another classic - Fibonacci:

```scots
dae fibonacci(n) {
    gin n <= 1 {
        gie n
    }
    gie fibonacci(n - 1) + fibonacci(n - 2)
}

fer i in 0..10 {
    blether f"fib({i}) = {fibonacci(i)}"
}
```

## Higher-Order Functions

Functions can accept other functions as parameters:

```scots
dae apply_twice(value, func) {
    gie func(func(value))
}

dae double(x) {
    gie x * 2
}

blether apply_twice(5, double)  # 20 (5 → 10 → 20)
```

Functions can also return functions:

```scots
dae make_multiplier(factor) {
    gie |x| x * factor
}

ken triple = make_multiplier(3)
blether triple(7)  # 21
```

## Lambdas (Anonymous Functions)

Short functions using pipe syntax:

```scots
# Single parameter
ken square = |x| x * x
blether square(5)  # 25

# Multiple parameters
ken add = |a, b| a + b
blether add(3, 4)  # 7

# No parameters
ken greet = || "Hullo!"
blether greet()  # "Hullo!"
```

Lambdas are great with higher-order functions:

```scots
ken numbers = [1, 2, 3, 4, 5]

ken doubled = gaun(numbers, |x| x * 2)
blether doubled  # [2, 4, 6, 8, 10]

ken evens = sieve(numbers, |x| x % 2 == 0)
blether evens  # [2, 4]
```

## Closures

Inner functions capture variables from their enclosing scope:

```scots
dae counter() {
    ken count = 0
    gie || {
        count = count + 1
        gie count
    }
}

ken my_counter = counter()
blether my_counter()  # 1
blether my_counter()  # 2
blether my_counter()  # 3
```

## Function Scope

Variables defined inside a function are local:

```scots
ken x = "outer"

dae test() {
    ken x = "inner"
    blether x  # "inner"
}

test()
blether x  # "outer"
```

Functions can access outer variables:

```scots
ken greeting = "Hullo"

dae say_hello(name) {
    blether f"{greeting}, {name}!"  # Uses outer 'greeting'
}

say_hello("Hamish")  # "Hullo, Hamish!"
```

## Practical Examples

### Validation Function

```scots
dae validate_age(age) {
    gin age < 0 {
        gie {"valid": nae, "message": "Age cannae be negative"}
    }
    gin age > 150 {
        gie {"valid": nae, "message": "That's awfy auld!"}
    }
    gie {"valid": aye, "message": "Age is braw"}
}

ken result = validate_age(25)
gin result["valid"] {
    blether "Valid age!"
} ither {
    blether result["message"]
}
```

### List Processing

```scots
dae find_max(list) {
    gin len(list) == 0 {
        gie naething
    }
    ken maximum = list[0]
    fer item in list {
        gin item > maximum {
            maximum = item
        }
    }
    gie maximum
}

blether find_max([3, 7, 2, 9, 4])  # 9
```

### Memoization

```scots
ken cache = {}

dae fib_memo(n) {
    ken key = tae_string(n)
    gin dict_has(cache, key) {
        gie cache[key]
    }

    ken result = naething
    gin n <= 1 {
        result = n
    } ither {
        result = fib_memo(n - 1) + fib_memo(n - 2)
    }

    cache[key] = result
    gie result
}

# Much faster for large n
blether fib_memo(30)  # 832040
```

## Function Best Practices

1. **Single responsibility**: Each function should do one thing well
2. **Descriptive names**: Use names that describe what the function does
3. **Keep functions short**: If a function is too long, break it up
4. **Document complex functions**: Add comments for non-obvious logic

```scots
# Good: Clear, focused function
dae calculate_average(numbers) {
    gin len(numbers) == 0 {
        gie 0
    }
    gie sumaw(numbers) / len(numbers)
}

# Good: Descriptive parameter names
dae format_price(amount, currency = "GBP") {
    gie f"{currency} {amount}"
}
```

## Exercises

1. **Palindrome Checker**: Write a function that checks if a string is a palindrome

2. **Power Function**: Write a recursive function to calculate x^n

3. **List Filter**: Write a function that filters a list based on a condition function

<details>
<summary>Solutions</summary>

```scots
# 1. Palindrome Checker
dae is_palindrome(text) {
    ken clean = lower(text)
    ken reversed_text = reverse(clean)
    gie clean == reversed_text
}

blether is_palindrome("radar")   # aye
blether is_palindrome("hello")   # nae

# 2. Power Function
dae power(x, n) {
    gin n == 0 {
        gie 1
    }
    gin n < 0 {
        gie 1 / power(x, -n)
    }
    gie x * power(x, n - 1)
}

blether power(2, 10)  # 1024
blether power(3, 4)   # 81

# 3. List Filter
dae my_filter(list, condition) {
    ken result = []
    fer item in list {
        gin condition(item) {
            shove(result, item)
        }
    }
    gie result
}

ken nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
ken evens = my_filter(nums, |x| x % 2 == 0)
blether evens  # [2, 4, 6, 8, 10]
```

</details>

## Next Steps

Learn about [data structures](./04-data-structures.md) to organize and manipulate collections of data.

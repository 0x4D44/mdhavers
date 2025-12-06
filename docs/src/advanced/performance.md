# Performance Tips

Optimize your mdhavers programs for speed and efficiency.

## General Principles

### 1. Avoid Repeated Work

Cache expensive computations:

```scots
# Bad: Recalculates len() each iteration
fer i in 0..len(my_list) {
    gin i < len(my_list) / 2 {
        # ...
    }
}

# Good: Calculate once
ken length = len(my_list)
ken half = length / 2
fer i in 0..length {
    gin i < half {
        # ...
    }
}
```

### 2. Use Built-in Functions

Built-in functions are implemented in Rust and are faster than equivalent mdhavers code:

```scots
# Bad: Manual sum
ken total = 0
fer n in numbers {
    total = total + n
}

# Good: Use built-in
ken total = sumaw(numbers)
```

### 3. Prefer Early Exit

Return early to avoid unnecessary work:

```scots
# Bad: Checks all items even after finding result
dae find_item(list, target) {
    ken found = naething
    fer item in list {
        gin item == target {
            found = item
        }
    }
    gie found
}

# Good: Return immediately when found
dae find_item(list, target) {
    fer item in list {
        gin item == target {
            gie item
        }
    }
    gie naething
}
```

## Data Structure Choices

### Lists vs Dictionaries

Use dictionaries for lookups, lists for sequences:

```scots
# Bad: O(n) lookup
ken users = [
    {"id": 1, "name": "Alice"},
    {"id": 2, "name": "Bob"},
    {"id": 3, "name": "Charlie"}
]

dae find_user(id) {
    fer user in users {
        gin user["id"] == id {
            gie user
        }
    }
    gie naething
}

# Good: O(1) lookup
ken users_by_id = {
    1: {"id": 1, "name": "Alice"},
    2: {"id": 2, "name": "Bob"},
    3: {"id": 3, "name": "Charlie"}
}

dae find_user(id) {
    gie dict_get(users_by_id, id, naething)
}
```

### String Building

Avoid repeated concatenation in loops:

```scots
# Bad: Creates many intermediate strings
ken result = ""
fer i in 0..1000 {
    result = result + tae_string(i) + ","
}

# Good: Build list, join at end
ken parts = []
fer i in 0..1000 {
    shove(parts, tae_string(i))
}
ken result = join(parts, ",")
```

## Loop Optimization

### Minimize Work Inside Loops

```scots
# Bad: Calls len() every iteration
fer i in 0..len(items) {
    ken length = len(items)  # Redundant!
    blether f"{i} of {length}"
}

# Good: Calculate outside
ken length = len(items)
fer i in 0..length {
    blether f"{i} of {length}"
}
```

### Use break When Possible

```scots
# Bad: Continues checking after condition met
dae has_negative(numbers) {
    ken found = nae
    fer n in numbers {
        gin n < 0 {
            found = aye
        }
    }
    gie found
}

# Good: Exit immediately
dae has_negative(numbers) {
    fer n in numbers {
        gin n < 0 {
            gie aye
        }
    }
    gie nae
}
```

## Memoization

Cache results of expensive function calls:

```scots
ken fib_cache = {}

dae fibonacci(n) {
    ken key = tae_string(n)

    gin dict_has(fib_cache, key) {
        gie fib_cache[key]
    }

    ken result = naething
    gin n <= 1 {
        result = n
    } ither {
        result = fibonacci(n - 1) + fibonacci(n - 2)
    }

    fib_cache[key] = result
    gie result
}

# Much faster for large n
blether fibonacci(40)
```

### Generic Memoization

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

# Usage
ken fast_fib = memoize(|n| {
    gin n <= 1 { gie n }
    gie fast_fib(n - 1) + fast_fib(n - 2)
})
```

## Algorithm Choices

### Choose Appropriate Algorithms

```scots
# Bad: O(n^2) - checking all pairs
dae has_duplicate_slow(list) {
    fer i in 0..len(list) {
        fer j in (i + 1)..len(list) {
            gin list[i] == list[j] {
                gie aye
            }
        }
    }
    gie nae
}

# Good: O(n) - use a set
dae has_duplicate(list) {
    ken seen = {}
    fer item in list {
        ken key = tae_string(item)
        gin dict_has(seen, key) {
            gie aye
        }
        seen[key] = aye
    }
    gie nae
}
```

### Binary Search for Sorted Data

```scots
dae binary_search(sorted_list, target) {
    ken low = 0
    ken high = len(sorted_list) - 1

    whiles low <= high {
        ken mid = (low + high) / 2
        ken mid_val = sorted_list[mid]

        gin mid_val == target {
            gie mid
        } ither gin mid_val < target {
            low = mid + 1
        } ither {
            high = mid - 1
        }
    }

    gie -1  # Not found
}
```

## Benchmarking

Use timing functions to measure performance:

```scots
ken start = tick()

# Code to benchmark
fer i in 0..10000 {
    ken x = i * i
}

ken elapsed = tick() - start
blether f"Elapsed: {elapsed / 1000000} ms"
```

### Comparing Approaches

```scots
dae benchmark(name, func, iterations) {
    ken start = tick()
    fer i in 0..iterations {
        func()
    }
    ken elapsed = (tick() - start) / 1000000
    blether f"{name}: {elapsed}ms for {iterations} iterations"
}

dae slow_approach() {
    ken result = ""
    fer i in 0..100 {
        result = result + "x"
    }
}

dae fast_approach() {
    ken parts = []
    fer i in 0..100 {
        shove(parts, "x")
    }
    ken result = join(parts, "")
}

benchmark("Slow", slow_approach, 100)
benchmark("Fast", fast_approach, 100)
```

## Common Pitfalls

### 1. Recursive vs Iterative

Recursion can be elegant but may be slower and risk stack overflow:

```scots
# Recursive (can overflow for large n)
dae factorial_recursive(n) {
    gin n <= 1 { gie 1 }
    gie n * factorial_recursive(n - 1)
}

# Iterative (safer, often faster)
dae factorial_iterative(n) {
    ken result = 1
    fer i in 2..(n + 1) {
        result = result * i
    }
    gie result
}
```

### 2. Unnecessary Type Conversions

```scots
# Bad: Converting to string just for comparison
gin tae_string(x) == tae_string(y) {
    # ...
}

# Good: Compare directly
gin x == y {
    # ...
}
```

### 3. Modifying Lists While Iterating

```scots
# Bad: Can cause issues
fer item in my_list {
    gin should_remove(item) {
        # Removing while iterating - dangerous!
    }
}

# Good: Filter to create new list
ken my_list = sieve(my_list, |item| nae should_remove(item))
```

## Summary

1. **Profile first** - Find where time is actually spent
2. **Use built-ins** - They're optimized in Rust
3. **Choose right data structures** - Dicts for lookups, lists for sequences
4. **Minimize loop work** - Calculate constants outside loops
5. **Exit early** - Return as soon as you have the answer
6. **Memoize** - Cache expensive computations
7. **Avoid allocations** - Reuse objects where possible

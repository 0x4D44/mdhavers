# Destructuring

Unpack lists and strings elegantly with destructuring assignment.

## Basic Destructuring

Extract elements from a list into variables:

```scots
ken [a, b, c] = [1, 2, 3]

blether a  # 1
blether b  # 2
blether c  # 3
```

## Destructuring Strings

Strings can be destructured into characters:

```scots
ken [first, second, third] = "ABC"

blether first   # "A"
blether second  # "B"
blether third   # "C"
```

## Rest Pattern (...)

Capture remaining elements with `...`:

### Rest at End

```scots
ken [first, ...rest] = [1, 2, 3, 4, 5]

blether first  # 1
blether rest   # [2, 3, 4, 5]
```

### Rest at Beginning

```scots
ken [...init, last] = [1, 2, 3, 4, 5]

blether init  # [1, 2, 3, 4]
blether last  # 5
```

### Rest in Middle

```scots
ken [head, ...middle, tail] = [1, 2, 3, 4, 5]

blether head    # 1
blether middle  # [2, 3, 4]
blether tail    # 5
```

## Ignoring Values with _

Use `_` to skip values you don't need:

```scots
ken [_, second, _] = ["skip", "keep", "skip"]

blether second  # "keep"
```

```scots
ken [first, _, _, fourth] = [1, 2, 3, 4]

blether first   # 1
blether fourth  # 4
```

## Combining Patterns

Mix rest and ignore patterns:

```scots
ken [_, ...middle, _] = [1, 2, 3, 4, 5]

blether middle  # [2, 3, 4]
```

## Function Returns

Destructuring works great with functions that return lists:

```scots
dae get_bounds(list) {
    ken sorted_list = sort(list)
    gie [heid(sorted_list), bum(sorted_list)]
}

ken [minimum, maximum] = get_bounds([5, 2, 8, 1, 9])

blether minimum  # 1
blether maximum  # 9
```

```scots
dae divide_with_remainder(a, b) {
    gie [a / b, a % b]
}

ken [quotient, remainder] = divide_with_remainder(17, 5)

blether quotient   # 3
blether remainder  # 2
```

## Swapping Variables

Elegant variable swap:

```scots
ken a = 1
ken b = 2

ken [a, b] = [b, a]

blether a  # 2
blether b  # 1
```

## Processing Pairs

```scots
ken coordinates = [[1, 2], [3, 4], [5, 6]]

fer coord in coordinates {
    ken [x, y] = coord
    blether f"Point: ({x}, {y})"
}
```

## Working with Key-Value Pairs

```scots
ken person = {"name": "Hamish", "age": 30}

fer pair in items(person) {
    ken [key, value] = pair
    blether f"{key}: {value}"
}
```

## Practical Examples

### Parsing Coordinates

```scots
dae parse_point(text) {
    # Expects "x,y" format
    ken parts = split(text, ",")
    ken [x_str, y_str] = parts
    gie [tae_int(x_str), tae_int(y_str)]
}

ken [x, y] = parse_point("10,20")
blether f"x={x}, y={y}"  # "x=10, y=20"
```

### Head and Tail Recursion

```scots
dae sum_list(list) {
    gin len(list) == 0 {
        gie 0
    }
    ken [first, ...rest] = list
    gie first + sum_list(rest)
}

blether sum_list([1, 2, 3, 4, 5])  # 15
```

### Multiple Assignment

```scots
dae calculate_stats(numbers) {
    ken total = sumaw(numbers)
    ken count = len(numbers)
    ken avg = total / count
    ken sorted_nums = sort(numbers)
    ken min_val = heid(sorted_nums)
    ken max_val = bum(sorted_nums)

    gie [total, avg, min_val, max_val]
}

ken [sum, mean, minimum, maximum] = calculate_stats([10, 20, 30, 40, 50])

blether f"Sum: {sum}"       # "Sum: 150"
blether f"Mean: {mean}"     # "Mean: 30"
blether f"Min: {minimum}"   # "Min: 10"
blether f"Max: {maximum}"   # "Max: 50"
```

### RGB Color

```scots
dae hex_to_rgb(hex) {
    # Parse "#RRGGBB" to [R, G, B]
    ken r = fae_hex(scran(hex, 1, 3))
    ken g = fae_hex(scran(hex, 3, 5))
    ken b = fae_hex(scran(hex, 5, 7))
    gie [r, g, b]
}

ken [red, green, blue] = hex_to_rgb("#FF8040")

blether f"R: {red}"    # "R: 255"
blether f"G: {green}"  # "G: 128"
blether f"B: {blue}"   # "B: 64"
```

## Edge Cases

### Empty Rest

```scots
ken [first, ...rest] = [1]

blether first  # 1
blether rest   # [] (empty list)
```

### Single Element

```scots
ken [only] = [42]
blether only  # 42
```

### Not Enough Elements

If there aren't enough elements, you'll get an error:

```scots
# This will error!
ken [a, b, c] = [1, 2]
# Error: Not enough elements to destructure
```

Use rest pattern to handle variable lengths:

```scots
ken [a, ...rest] = [1, 2]  # Works fine
blether a     # 1
blether rest  # [2]
```

## Best Practices

1. **Match expected length**: Use the right number of variables

2. **Use rest for flexibility**: When length might vary

3. **Name variables meaningfully**: `ken [x, y]` better than `ken [a, b]`

4. **Don't over-nest**: Keep destructuring simple

```scots
# Good: Clear and simple
ken [name, age] = get_person_info()

# Okay: Use _ for unneeded values
ken [_, score, _] = get_full_record()

# Good: Rest for variable length
ken [first, ...others] = get_items()
```

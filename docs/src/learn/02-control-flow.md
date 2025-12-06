# Control Flow

Make decisions and repeat actions with conditions and loops.

## Conditionals with gin/ither

Use `gin` (Scots for "if") to execute code conditionally:

```scots
ken temperature = 15

gin temperature < 10 {
    blether "It's pure Baltic oot there!"
}
```

Add `ither` (else) for alternative branches:

```scots
ken age = 16

gin age >= 18 {
    blether "Ye can vote!"
} ither {
    blether "Nae votin' fer ye yet."
}
```

Chain multiple conditions with `ither gin`:

```scots
ken score = 75

gin score >= 90 {
    blether "Braw! An A!"
} ither gin score >= 80 {
    blether "No bad! A B."
} ither gin score >= 70 {
    blether "Ye passed wi' a C."
} ither gin score >= 60 {
    blether "Just scraped a D."
} ither {
    blether "Och, ye failed."
}
```

### Comparison Operators

| Operator | Meaning |
|----------|---------|
| `==` | Equal to |
| `!=` | Not equal to |
| `<` | Less than |
| `>` | Greater than |
| `<=` | Less than or equal |
| `>=` | Greater than or equal |

### Logical Operators

Combine conditions with `an` (and) and `or`:

```scots
ken age = 25
ken has_ticket = aye

gin age >= 18 an has_ticket {
    blether "Welcome tae the concert!"
}

ken is_member = nae
ken has_voucher = aye

gin is_member or has_voucher {
    blether "Ye get a discount!"
}
```

Negate conditions with `nae`:

```scots
ken is_raining = nae

gin nae is_raining {
    blether "Nae need fer a brolly!"
}
```

### Nested Conditions

Conditions can be nested:

```scots
ken age = 30
ken has_license = aye

gin age >= 18 {
    gin has_license {
        blether "Ye can drive!"
    } ither {
        blether "Ye're auld enough but need a license."
    }
} ither {
    blether "Ye're too young tae drive."
}
```

## Ternary Expressions

For simple conditional assignments, use `gin...than...ither`:

```scots
ken age = 20
ken status = gin age >= 18 than "adult" ither "minor"
blether status  # "adult"

ken x = 10
ken y = 20
ken bigger = gin x > y than x ither y
blether bigger  # 20
```

## While Loops with whiles

Repeat while a condition is true:

```scots
ken count = 1

whiles count <= 5 {
    blether f"Count: {count}"
    count = count + 1
}
```

Output:
```
Count: 1
Count: 2
Count: 3
Count: 4
Count: 5
```

### Infinite Loops (Be Careful!)

```scots
# This runs forever - press Ctrl+C to stop!
whiles aye {
    blether "Forever!"
}
```

### Loop Until Condition Met

```scots
ken guess = 0
ken target = 7

whiles guess != target {
    guess = tae_int(speir "Guess a number: ")
    gin guess < target {
        blether "Too low!"
    } ither gin guess > target {
        blether "Too high!"
    }
}
blether "Ye got it!"
```

## For Loops with fer

### Loop Over a Range

Use `..` to create a range:

```scots
# 1 to 5 (exclusive of 6)
fer i in 1..6 {
    blether i
}
```

Output:
```
1
2
3
4
5
```

### Loop Over a List

```scots
ken cities = ["Edinburgh", "Glasgow", "Aberdeen", "Dundee"]

fer city in cities {
    blether f"Visit {city}!"
}
```

### Loop Over a String

```scots
ken word = "Scotland"

fer letter in word {
    blether letter
}
```

### Loop Over Dictionary Keys

```scots
ken person = {"name": "Hamish", "age": 30, "city": "Glasgow"}

fer key in keys(person) {
    blether f"{key}: {person[key]}"
}
```

### Loop with Index

Use `enumerate` pattern manually:

```scots
ken fruits = ["apple", "banana", "cherry"]
ken i = 0

fer fruit in fruits {
    blether f"{i}: {fruit}"
    i = i + 1
}
```

## Break and Continue

### brak (break)

Exit a loop early:

```scots
fer i in 1..100 {
    gin i > 5 {
        blether "Stopping!"
        brak
    }
    blether i
}
```

Output:
```
1
2
3
4
5
Stopping!
```

### haud (continue)

Skip to the next iteration:

```scots
# Print only odd numbers
fer i in 1..11 {
    gin i % 2 == 0 {
        haud  # Skip even numbers
    }
    blether i
}
```

Output:
```
1
3
5
7
9
```

### Combining break and continue

```scots
ken numbers = [1, -2, 3, -4, 5, -6, 0, 7]

fer n in numbers {
    gin n == 0 {
        blether "Found zero, stopping!"
        brak
    }
    gin n < 0 {
        haud  # Skip negatives
    }
    blether f"Positive: {n}"
}
```

## Nested Loops

Loops within loops:

```scots
fer i in 1..4 {
    fer j in 1..4 {
        blether f"{i} x {j} = {i * j}"
    }
    blether "---"
}
```

### Multiplication Table

```scots
blether "Times Table"
blether "============"

fer i in 1..6 {
    ken row = ""
    fer j in 1..6 {
        ken product = i * j
        row = row + tae_string(product) + "\t"
    }
    blether row
}
```

## Practical Examples

### FizzBuzz

The classic programming challenge:

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

### Find Maximum

```scots
ken numbers = [34, 67, 12, 89, 45, 23]
ken maximum = numbers[0]

fer num in numbers {
    gin num > maximum {
        maximum = num
    }
}

blether f"Maximum: {maximum}"  # Maximum: 89
```

### Count Occurrences

```scots
ken text = "banana"
ken target = "a"
ken count = 0

fer char in text {
    gin char == target {
        count = count + 1
    }
}

blether f"'{target}' appears {count} times"  # 'a' appears 3 times
```

## Exercises

1. **Sum of Even Numbers**: Sum all even numbers from 1 to 100

2. **Prime Checker**: Check if a number is prime

3. **Countdown**: Print a countdown from 10 to 1, then "Blast off!"

<details>
<summary>Solutions</summary>

```scots
# 1. Sum of Even Numbers
ken total = 0
fer i in 1..101 {
    gin i % 2 == 0 {
        total = total + i
    }
}
blether f"Sum of evens: {total}"  # 2550

# 2. Prime Checker
ken n = 17
ken is_prime = aye

gin n < 2 {
    is_prime = nae
} ither {
    fer i in 2..n {
        gin n % i == 0 {
            is_prime = nae
            brak
        }
    }
}

gin is_prime {
    blether f"{n} is prime!"
} ither {
    blether f"{n} is nae prime."
}

# 3. Countdown
fer i in reverse(1..11) {
    blether i
}
blether "Blast off!"
```

</details>

## Next Steps

Now that you can control program flow, learn about [functions](./03-functions.md) to organize your code into reusable pieces.

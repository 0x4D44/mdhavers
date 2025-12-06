# Operator Overloading

Define custom behavior for operators with your classes.

## Overview

mdhavers lets you define how operators work with your custom types by implementing special methods in your classes. These methods have Scottish names that describe what the operator does.

## Arithmetic Operators

### Addition: __pit_thegither__

"Pit thegither" means "put together":

```scots
kin Vector {
    dae init(x, y) {
        masel.x = x
        masel.y = y
    }

    dae __pit_thegither__(that) {
        gie Vector(masel.x + that.x, masel.y + that.y)
    }

    dae to_string() {
        gie f"({masel.x}, {masel.y})"
    }
}

ken v1 = Vector(3, 4)
ken v2 = Vector(1, 2)
ken v3 = v1 + v2

blether v3.to_string()  # "(4, 6)"
```

### Subtraction: __tak_awa__

"Tak awa" means "take away":

```scots
kin Money {
    dae init(amount) {
        masel.amount = amount
    }

    dae __pit_thegither__(that) {
        gie Money(masel.amount + that.amount)
    }

    dae __tak_awa__(that) {
        gie Money(masel.amount - that.amount)
    }
}

ken balance = Money(100)
ken expense = Money(30)
ken remaining = balance - expense

blether remaining.amount  # 70
```

### Multiplication: __times__

```scots
kin Vector {
    dae init(x, y) {
        masel.x = x
        masel.y = y
    }

    dae __times__(scalar) {
        gie Vector(masel.x * scalar, masel.y * scalar)
    }
}

ken v = Vector(3, 4)
ken scaled = v * 3

blether scaled.x  # 9
blether scaled.y  # 12
```

### Division: __pairt__

"Pairt" means "divide" or "part":

```scots
kin Fraction {
    dae init(num, den) {
        masel.num = num
        masel.den = den
    }

    dae __pairt__(that) {
        # Dividing fractions: multiply by reciprocal
        gie Fraction(masel.num * that.den, masel.den * that.num)
    }
}

ken half = Fraction(1, 2)
ken quarter = Fraction(1, 4)
ken result = half / quarter  # 1/2 ÷ 1/4 = 2

blether f"{result.num}/{result.den}"  # "2/1"
```

### Modulo: __lave__

"Lave" means "remainder" or "what's left":

```scots
kin Integer {
    dae init(value) {
        masel.value = value
    }

    dae __lave__(that) {
        gie Integer(masel.value % that.value)
    }
}

ken a = Integer(17)
ken b = Integer(5)
ken remainder = a % b

blether remainder.value  # 2
```

## Comparison Operators

### Equality: __same_as__

```scots
kin Point {
    dae init(x, y) {
        masel.x = x
        masel.y = y
    }

    dae __same_as__(that) {
        gie masel.x == that.x an masel.y == that.y
    }
}

ken p1 = Point(3, 4)
ken p2 = Point(3, 4)
ken p3 = Point(1, 2)

blether p1 == p2  # aye
blether p1 == p3  # nae
```

### Inequality: __differs_fae__

"Differs fae" means "different from":

```scots
kin Point {
    dae init(x, y) {
        masel.x = x
        masel.y = y
    }

    dae __same_as__(that) {
        gie masel.x == that.x an masel.y == that.y
    }

    dae __differs_fae__(that) {
        gie nae masel.__same_as__(that)
    }
}

ken p1 = Point(3, 4)
ken p2 = Point(1, 2)

blether p1 != p2  # aye
```

### Less Than: __wee_er__

"Wee-er" means "smaller":

```scots
kin Temperature {
    dae init(celsius) {
        masel.celsius = celsius
    }

    dae __wee_er__(that) {
        gie masel.celsius < that.celsius
    }
}

ken cold = Temperature(-5)
ken warm = Temperature(20)

blether cold < warm  # aye
blether warm < cold  # nae
```

### Greater Than: __muckle_er__

"Muckle-er" means "bigger":

```scots
kin Score {
    dae init(value) {
        masel.value = value
    }

    dae __muckle_er__(that) {
        gie masel.value > that.value
    }
}

ken high = Score(95)
ken low = Score(60)

blether high > low  # aye
```

### Less or Equal: __wee_er_or_same__

```scots
kin Version {
    dae init(major, minor) {
        masel.major = major
        masel.minor = minor
    }

    dae __wee_er_or_same__(that) {
        gin masel.major < that.major {
            gie aye
        }
        gin masel.major == that.major {
            gie masel.minor <= that.minor
        }
        gie nae
    }
}

ken v1 = Version(1, 0)
ken v2 = Version(1, 5)
ken v3 = Version(2, 0)

blether v1 <= v2  # aye
blether v2 <= v3  # aye
blether v3 <= v1  # nae
```

### Greater or Equal: __muckle_er_or_same__

```scots
kin Size {
    dae init(bytes) {
        masel.bytes = bytes
    }

    dae __muckle_er_or_same__(that) {
        gie masel.bytes >= that.bytes
    }
}

ken big_file = Size(1000000)
ken small_file = Size(100)

blether big_file >= small_file  # aye
```

## Complete Reference

| Method | Operator | Scots Meaning |
|--------|----------|---------------|
| `__pit_thegither__(that)` | `+` | Put together (add) |
| `__tak_awa__(that)` | `-` | Take away (subtract) |
| `__times__(that)` | `*` | Times (multiply) |
| `__pairt__(that)` | `/` | Part (divide) |
| `__lave__(that)` | `%` | Remainder (what's left) |
| `__same_as__(that)` | `==` | Same as (equal) |
| `__differs_fae__(that)` | `!=` | Differs from (not equal) |
| `__wee_er__(that)` | `<` | Smaller (less than) |
| `__wee_er_or_same__(that)` | `<=` | Smaller or same |
| `__muckle_er__(that)` | `>` | Bigger (greater than) |
| `__muckle_er_or_same__(that)` | `>=` | Bigger or same |

## Complete Example: Complex Numbers

```scots
kin Complex {
    dae init(real, imag) {
        masel.real = real
        masel.imag = imag
    }

    # Addition
    dae __pit_thegither__(that) {
        gie Complex(
            masel.real + that.real,
            masel.imag + that.imag
        )
    }

    # Subtraction
    dae __tak_awa__(that) {
        gie Complex(
            masel.real - that.real,
            masel.imag - that.imag
        )
    }

    # Multiplication: (a+bi)(c+di) = (ac-bd) + (ad+bc)i
    dae __times__(that) {
        gie Complex(
            masel.real * that.real - masel.imag * that.imag,
            masel.real * that.imag + masel.imag * that.real
        )
    }

    # Equality
    dae __same_as__(that) {
        gie masel.real == that.real an masel.imag == that.imag
    }

    # Magnitude (for comparison)
    dae magnitude() {
        gie sqrt(masel.real * masel.real + masel.imag * masel.imag)
    }

    dae __wee_er__(that) {
        gie masel.magnitude() < that.magnitude()
    }

    dae to_string() {
        gin masel.imag >= 0 {
            gie f"{masel.real} + {masel.imag}i"
        }
        gie f"{masel.real} - {abs(masel.imag)}i"
    }
}

ken c1 = Complex(3, 4)
ken c2 = Complex(1, 2)

blether (c1 + c2).to_string()  # "4 + 6i"
blether (c1 - c2).to_string()  # "2 + 2i"
blether (c1 * c2).to_string()  # "-5 + 10i"
blether c1 == c1  # aye
blether c1 < c2   # nae (magnitude 5 vs ~2.24)
```

## Best Practices

1. **Be consistent**: If you implement `__same_as__`, also implement `__differs_fae__`

2. **Return the right type**: Arithmetic operators should return a new instance of your class

3. **Don't mutate**: Operations should create new objects, not modify existing ones

4. **Handle edge cases**: Consider what happens with zero, negative values, etc.

```scots
# Good: Returns new instance
dae __pit_thegither__(that) {
    gie Vector(masel.x + that.x, masel.y + that.y)
}

# Bad: Mutates self
dae __pit_thegither__(that) {
    masel.x = masel.x + that.x
    masel.y = masel.y + that.y
    gie masel  # Bad practice!
}
```

## Chaining Operations

With proper operator overloading, you can chain operations naturally:

```scots
ken v1 = Vector(1, 0)
ken v2 = Vector(0, 1)
ken v3 = Vector(2, 2)

# This works because + returns a Vector
ken result = v1 + v2 + v3
blether result.to_string()  # "(3, 3)"

# Or with mixed operations
ken scaled = (v1 + v2) * 3
blether scaled.to_string()  # "(3, 3)"
```

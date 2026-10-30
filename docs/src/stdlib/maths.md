# maths

Everyday arithmetic, number theory, simple statistics, basic geometry and sequence helpers, all in plain mdhavers. Works on every backend (no native built-ins beyond `sqrt`, `sort`, `sumaw`, `keys` and friends).

Import: `fetch "stdlib/maths"`

Importing prints `Maths module loaded! Numbers are braw!`. Division of two integers is integer division, so `mean`, `median`, `variance` and `std_dev` truncate when the inputs are all integers. Pass floats (for example `[1.0, 2.0]`) when you want exact averages. Helpers starting with `_` are internal and not documented.

## Constants

| Name | Value |
|------|-------|
| `PI` | 3.14159265358979... |
| `E` | 2.71828182845904... |
| `TAU` | 2 * PI |
| `PHI` | golden ratio, 1.6180339887... |

## Functions

### sign(n)
Returns -1, 0 or 1 for a negative, zero or positive number.

### pooer(base, exp)
`base` raised to an integer `exp`. `exp == 0` gives 1; a negative `exp` gives `1 / pooer(base, -exp)` (so integer bases with a negative exponent truncate, for example `pooer(2, -1)` is `0`; use `2.0`).

### factorial(n)
`n!`, recursive. Fails a `mak_siccar` for negative input: `Cannae dae factorial o' negative numbers!`. `factorial(0)` and `factorial(1)` are 1.

### gcd(a, b)
Greatest common divisor by Euclid, on absolute values.

### lcm(a, b)
Least common multiple. Returns 0 if either argument is 0.

### is_prime(n)
`aye` if `n` is prime, otherwise `nae`. Numbers below 2 are not prime.

### prime_factors(n)
List of prime factors in ascending order with repeats, for example `prime_factors(12)` is `[2, 2, 3]`.

### primes_up_tae(n)
List of all primes from 2 up to and including `n`. Returns `[]` when `n < 2`. (It tests each number with `is_prime`; it is not a true sieve.)

### fib_sequence(n)
First `n` Fibonacci numbers starting `0, 1`. Returns `[]` for `n <= 0`.

### fib(n)
The `n`th Fibonacci number, iteratively (`fib(0)` is 0, `fib(1)` is 1).

### fibonacci(n)
Same result as `fib` but recursive and therefore slow for large `n`.

### mean(nums)
Average of a list. Fails a `mak_siccar` on an empty list: `Cannae find mean o' empty list!`.

### median(nums)
Middle value of the sorted list, or the average of the two middle values. Empty list: `Cannae find median o' empty list!`.

### mode(nums)
Most frequent value (first one reached wins a tie). Empty list: `Cannae find mode o' empty list!`. Values are keyed by string and converted back with `tae_int`, so it works for integers only.

### variance(nums)
Population variance. Empty list: `Cannae find variance o' empty list!`.

### std_dev(nums)
Square root of `variance(nums)`; fails the same way on an empty list.

### tae_radians(degrees)
Degrees to radians.

### tae_degrees(radians)
Radians to degrees.

### distance(x1, y1, x2, y2)
Straight-line distance between two points.

### circle_area(radius)
`PI * r * r`.

### circle_circumference(radius)
`2 * PI * r`.

### triangle_area(base, height)
`base * height / 2`.

### triangle_area_heron(a, b, c)
Triangle area from three side lengths (Heron's formula).

### range_step(start, finish, step)
List from `start` towards `finish` (exclusive) in steps. A positive `step` counts up, any other step counts down while `i > finish`. A zero step counts as "down": it loops forever when `start > finish` and returns `[]` otherwise (`range_step(0, 5, 0)` is `[]`).

### repeat(val, n)
List of `n` copies of `val`.

### square(x)
`x * x`.

### cube(x)
`x * x * x`.

### clamp(val, min_val, max_val)
`val` limited to the range `[min_val, max_val]`.

## Example

```mdhavers
fetch "stdlib/maths"

blether gcd(48, 18)
blether prime_factors(60)
blether primes_up_tae(20)
blether fib(10)
blether median([5, 1, 3])
blether clamp(15, 0, 10)
```

## Compared with math

[maths](./maths.md) is the small, function-only toolkit: number theory, statistics, a few geometry formulas and sequences. [math](./math.md) is a geometry and linear-algebra library built on classes (`Vec2`, `Vec3`, `Mat2`, `Mat3`, shapes, curves, collision helpers). They do not clash if both are imported: `math` has `distance_2d` and a `Vec2.distance` method where `maths` has `distance`. Use `maths` for numbers, lists of numbers and formulas; use `math` when you need vectors, transforms or shapes.

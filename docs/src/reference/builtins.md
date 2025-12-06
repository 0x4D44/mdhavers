# Built-in Functions Reference

Complete reference of all built-in functions in mdhavers.

## Scots-Flavored Functions

These functions have distinctly Scottish names and personality!

### List Functions

| Function | Description | Example |
|----------|-------------|---------|
| `heid(list)` | Get first element (head) | `heid([1,2,3])` → `1` |
| `bum(list)` | Get last element (backside) | `bum([1,2,3])` → `3` |
| `tail(list)` | All but first element | `tail([1,2,3])` → `[2,3]` |
| `scran(x, start, end)` | Slice (grab food) | `scran([1,2,3,4], 1, 3)` → `[2,3]` |
| `slap(a, b)` | Concatenate (friendly slap) | `slap([1,2], [3,4])` → `[1,2,3,4]` |
| `sumaw(list)` | Sum all | `sumaw([1,2,3])` → `6` |
| `drap(list, n)` | Drop first n | `drap([1,2,3], 2)` → `[3]` |
| `tak(list, n)` | Take first n | `tak([1,2,3], 2)` → `[1,2]` |
| `grup(list, n)` | Group into chunks | `grup([1,2,3,4], 2)` → `[[1,2],[3,4]]` |
| `fankle(a, b)` | Interleave (tangle) | `fankle([1,2], [3,4])` → `[1,3,2,4]` |
| `stoater(list)` | Get max (great one) | `stoater([1,5,3])` → `5` |
| `dram(list)` | Random element (wee drink) | `dram([1,2,3])` → random |
| `birl(list, n)` | Rotate (spin) | `birl([1,2,3], 1)` → `[2,3,1]` |
| `sclaff(list)` | Deep flatten (hit flat) | `sclaff([[1,[2]],3])` → `[1,2,3]` |

### String Functions

| Function | Description | Example |
|----------|-------------|---------|
| `wheesht(str)` | Trim whitespace (be quiet) | `wheesht("  hi  ")` → `"hi"` |
| `scottify(str)` | Convert English to Scots | `scottify("hello")` → `"hullo"` |
| `stooshie(str)` | Shuffle characters (chaos) | `stooshie("hello")` → random |
| `blooter(str)` | Scramble randomly | `blooter("hello")` → random |
| `tattie_scone(s, n)` | Repeat with \| separator | `tattie_scone("la", 3)` → `"la\|la\|la"` |
| `sporran_fill(s, w, c)` | Center-pad string | `sporran_fill("hi", 6, "-")` → `"--hi--"` |
| `haggis_hunt(s, needle)` | Find all occurrences | `haggis_hunt("aa", "a")` → `[0,1]` |

### Validation Functions

| Function | Description | Example |
|----------|-------------|---------|
| `braw(x)` | Check if value is "good" | `braw("hi")` → `aye` |
| `clarty(list)` | Check for duplicates (dirty) | `clarty([1,1,2])` → `aye` |
| `dreich(str)` | Check if monotonous (dull) | `dreich("aaa")` → `aye` |
| `haverin(x)` | Check if empty/nonsense | `haverin("")` → `aye` |
| `scunner(x)` | Check if negative/empty | `scunner(-5)` → `aye` |
| `is_wee(x)` | Check if small | `is_wee(2)` → `aye` |
| `is_muckle(x)` | Check if large | `is_muckle(1000)` → `aye` |
| `crabbit(n)` | Check if negative (grumpy) | `crabbit(-5)` → `aye` |
| `cannie(x)` | Check if safe/valid | `cannie(5)` → `aye` |
| `glaikit(x)` | Check if empty/zero (silly) | `glaikit(0)` → `aye` |

### Expression Functions

| Function | Description | Example |
|----------|-------------|---------|
| `och(msg)` | Express disappointment | `och("no!")` → `"Och! no!"` |
| `jings(msg)` | Express surprise | `jings("wow")` → `"Jings! wow"` |
| `crivvens(msg)` | Express astonishment | `crivvens("!")` → `"Crivvens! !"` |
| `help_ma_boab(msg)` | Extreme surprise | `help_ma_boab("!")` |
| `roar(str)` | Shout (uppercase + !) | `roar("hello")` → `"HELLO!"` |
| `mutter(str)` | Whisper | `mutter("HI")` → `"...hi..."` |
| `bonnie(x)` | Decorate prettily | `bonnie("hi")` → `"~~~ hi ~~~"` |

### Utility Functions

| Function | Description | Example |
|----------|-------------|---------|
| `clype(x)` | Debug print with type (tell tales) | `clype(42)` → prints type info |
| `indices_o(x, val)` | Find all indices of value | `indices_o([1,2,1], 1)` → `[0,2]` |
| `braw_date(ts)` | Format date Scottish style | `braw_date(noo())` |
| `grup_up(list, fn)` | Group by function | `grup_up([1,2,3], \|x\| x%2)` |
| `pairt_by(list, fn)` | Partition by predicate | `pairt_by([1,2,3], \|x\| x>1)` |
| `pair_up(list)` | Create pairs | `pair_up([1,2,3,4])` → `[[1,2],[3,4]]` |
| `ceilidh(l1, l2)` | Interleave like dancers | `ceilidh([1,2],[3,4])` |

## Higher-Order Functions

| Function | Description | Example |
|----------|-------------|---------|
| `gaun(list, fn)` | Map (going over) | `gaun([1,2], \|x\| x*2)` → `[2,4]` |
| `sieve(list, fn)` | Filter | `sieve([1,2,3], \|x\| x>1)` → `[2,3]` |
| `tumble(list, init, fn)` | Reduce/fold | `tumble([1,2], 0, \|a,x\| a+x)` → `3` |
| `ilk(list, fn)` | For-each (each) | `ilk([1,2], print)` |

## Type Functions

| Function | Description | Example |
|----------|-------------|---------|
| `whit_kind(x)` | Get type name | `whit_kind(42)` → `"integer"` |
| `is_a(x, type)` | Check type | `is_a(42, "integer")` → `aye` |
| `tae_string(x)` | Convert to string | `tae_string(42)` → `"42"` |
| `tae_int(x)` | Convert to integer | `tae_int("42")` → `42` |
| `tae_float(x)` | Convert to float | `tae_float("3.14")` → `3.14` |
| `tae_bool(x)` | Convert to boolean | `tae_bool(1)` → `aye` |

## List Operations

| Function | Description | Example |
|----------|-------------|---------|
| `len(x)` | Length | `len([1,2,3])` → `3` |
| `shove(list, x)` | Append (push) | `shove([1,2], 3)` → `[1,2,3]` |
| `yank(list)` | Pop last | `yank([1,2,3])` → `3` |
| `sort(list)` | Sort ascending | `sort([3,1,2])` → `[1,2,3]` |
| `reverse(x)` | Reverse | `reverse([1,2,3])` → `[3,2,1]` |
| `contains(x, y)` | Check membership | `contains([1,2], 1)` → `aye` |
| `coont(x, y)` | Count occurrences | `coont([1,1,2], 1)` → `2` |
| `shuffle(list)` | Random shuffle | `shuffle([1,2,3])` |
| `jammy(min, max)` | Random int in range | `jammy(1, 10)` → random |

## String Operations

| Function | Description | Example |
|----------|-------------|---------|
| `upper(str)` | Uppercase | `upper("hello")` → `"HELLO"` |
| `lower(str)` | Lowercase | `lower("HELLO")` → `"hello"` |
| `split(str, delim)` | Split string | `split("a,b", ",")` → `["a","b"]` |
| `join(list, delim)` | Join to string | `join(["a","b"], "-")` → `"a-b"` |
| `pad_left(s, w, c)` | Left pad | `pad_left("5", 3, "0")` → `"005"` |
| `pad_right(s, w, c)` | Right pad | `pad_right("5", 3, "0")` → `"500"` |
| `center(s, w, c)` | Center pad | `center("hi", 6, "-")` → `"--hi--"` |
| `is_upper(s)` | All uppercase? | `is_upper("ABC")` → `aye` |
| `is_lower(s)` | All lowercase? | `is_lower("abc")` → `aye` |
| `swapcase(s)` | Swap case | `swapcase("HeLLo")` → `"hEllO"` |
| `strip_left(s, chars)` | Strip leading | `strip_left("xxhi", "x")` → `"hi"` |
| `strip_right(s, chars)` | Strip trailing | `strip_right("hixx", "x")` → `"hi"` |
| `replace_first(s, from, to)` | Replace first | `replace_first("aa", "a", "b")` → `"ba"` |
| `substr_between(s, start, end)` | Get between | `substr_between("<x>", "<", ">")` → `"x"` |

## Dictionary Operations

| Function | Description | Example |
|----------|-------------|---------|
| `keys(dict)` | Get keys | `keys({"a":1})` → `["a"]` |
| `values(dict)` | Get values | `values({"a":1})` → `[1]` |
| `items(dict)` | Get pairs | `items({"a":1})` → `[["a",1]]` |
| `dict_merge(d1, d2)` | Merge dicts | `dict_merge({"a":1}, {"b":2})` |
| `dict_get(d, key, default)` | Safe get | `dict_get({}, "x", 0)` → `0` |
| `dict_has(d, key)` | Key exists? | `dict_has({"a":1}, "a")` → `aye` |
| `dict_remove(d, key)` | Remove key | `dict_remove({"a":1}, "a")` |
| `dict_invert(d)` | Swap key/value | `dict_invert({"a":1})` → `{1:"a"}` |
| `fae_pairs(list)` | Create from pairs | `fae_pairs([["a",1]])` → `{"a":1}` |

## Math Functions

| Function | Description | Example |
|----------|-------------|---------|
| `abs(n)` | Absolute value | `abs(-5)` → `5` |
| `min(a, b)` | Minimum | `min(3, 5)` → `3` |
| `max(a, b)` | Maximum | `max(3, 5)` → `5` |
| `sqrt(n)` | Square root | `sqrt(16)` → `4` |
| `floor(n)` | Round down | `floor(3.7)` → `3` |
| `ceil(n)` | Round up | `ceil(3.2)` → `4` |
| `round(n)` | Round | `round(3.5)` → `4` |
| `pooer(x, y)` | Power | `pooer(2, 3)` → `8` |
| `sign(n)` | Sign (-1, 0, 1) | `sign(-5)` → `-1` |
| `clamp(n, min, max)` | Constrain | `clamp(15, 0, 10)` → `10` |
| `lerp(a, b, t)` | Interpolate | `lerp(0, 10, 0.5)` → `5` |
| `gcd(a, b)` | Greatest common divisor | `gcd(12, 8)` → `4` |
| `lcm(a, b)` | Least common multiple | `lcm(4, 6)` → `12` |
| `factorial(n)` | Factorial | `factorial(5)` → `120` |
| `is_even(n)` | Is even? | `is_even(4)` → `aye` |
| `is_odd(n)` | Is odd? | `is_odd(3)` → `aye` |
| `is_prime(n)` | Is prime? | `is_prime(7)` → `aye` |

### Trigonometry

| Function | Description |
|----------|-------------|
| `sin(n)` | Sine (radians) |
| `cos(n)` | Cosine |
| `tan(n)` | Tangent |
| `asin(n)` | Arc sine |
| `acos(n)` | Arc cosine |
| `atan(n)` | Arc tangent |
| `atan2(y, x)` | Two-argument arc tangent |
| `hypot(x, y)` | Hypotenuse |
| `degrees(rad)` | Radians to degrees |
| `radians(deg)` | Degrees to radians |

### Logarithms

| Function | Description |
|----------|-------------|
| `log(n)` | Natural logarithm |
| `log10(n)` | Base 10 logarithm |
| `exp(n)` | e^n |

### Constants

| Constant | Value |
|----------|-------|
| `PI` | 3.14159... |
| `E` | 2.71828... |
| `TAU` | 6.28318... (2π) |

## Bitwise Operations

| Function | Description | Example |
|----------|-------------|---------|
| `bit_an(a, b)` | AND | `bit_an(5, 3)` → `1` |
| `bit_or(a, b)` | OR | `bit_or(5, 3)` → `7` |
| `bit_xor(a, b)` | XOR | `bit_xor(5, 3)` → `6` |
| `bit_nae(n)` | NOT | `bit_nae(5)` → `-6` |
| `bit_shove_left(n, shift)` | Left shift | `bit_shove_left(1, 3)` → `8` |
| `bit_shove_right(n, shift)` | Right shift | `bit_shove_right(8, 2)` → `2` |
| `bit_coont(n)` | Popcount | `bit_coont(7)` → `3` |
| `tae_binary(n)` | To binary string | `tae_binary(5)` → `"101"` |
| `tae_hex(n)` | To hex string | `tae_hex(255)` → `"ff"` |
| `tae_octal(n)` | To octal string | `tae_octal(8)` → `"10"` |
| `fae_binary(s)` | From binary | `fae_binary("101")` → `5` |
| `fae_hex(s)` | From hex | `fae_hex("ff")` → `255` |

## Timing Functions

| Function | Description | Example |
|----------|-------------|---------|
| `noo()` | Current time (ms) | `noo()` → timestamp |
| `the_noo()` | Current timestamp | `the_noo()` |
| `tick()` | High-precision (ns) | `tick()` → nanoseconds |
| `bide(ms)` | Sleep (wait) | `bide(1000)` → sleeps 1s |
| `snooze(ms)` | Sleep | `snooze(500)` |

## File I/O

| Function | Description | Example |
|----------|-------------|---------|
| `scrieve(path, content)` | Write file | `scrieve("f.txt", "hi")` |
| `read_file(path)` | Read entire file | `read_file("f.txt")` |
| `read_lines(path)` | Read as lines | `read_lines("f.txt")` |
| `append_file(path, content)` | Append to file | `append_file("f.txt", "more")` |
| `file_exists(path)` | Check if exists | `file_exists("f.txt")` |

## List Statistics

| Function | Description | Example |
|----------|-------------|---------|
| `average(list)` | Mean | `average([1,2,3])` → `2` |
| `median(list)` | Median | `median([1,2,3])` → `2` |
| `product(list)` | Product | `product([2,3,4])` → `24` |
| `minaw(list)` | Minimum | `minaw([3,1,2])` → `1` |
| `maxaw(list)` | Maximum | `maxaw([3,1,2])` → `3` |
| `range_o(list)` | Range (max-min) | `range_o([1,5])` → `4` |

## Assertions

| Function | Description | Example |
|----------|-------------|---------|
| `assert(cond, msg)` | Assert true | `assert(x > 0, "must be positive")` |
| `assert_equal(a, b)` | Assert equal | `assert_equal(1+1, 2)` |
| `assert_nae_equal(a, b)` | Assert not equal | `assert_nae_equal(1, 2)` |
| `mak_siccar(cond, msg)` | Assert (Scots) | `mak_siccar(x > 0)` |

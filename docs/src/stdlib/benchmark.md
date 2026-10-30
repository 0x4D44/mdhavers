# benchmark

Performance-testing helpers: time a function, average it over many runs, record and print results, compare two functions, run a suite, crudely estimate memory use of lists and dicts, and check by doubling input sizes whether a function looks linear.

Import: `fetch "stdlib/benchmark"`

Uses the built-in `tick()` (a high-resolution clock) plus `whit_kind`, `keys` and `contains`. Loading the module prints `Benchmark module loaded! Time tae measure!`. Helpers starting with `_` are internal and not documented (this module has none).

Units: the source comments and `format_time` call the numbers microseconds, but the built-in `tick()` returns nanoseconds (about 1.79e18 now), so the values returned by `time_it` and `benchmark` are nanoseconds and `format_time` labels them `µs`. Treat results as relative numbers for comparing functions, not as absolute times. With integer arithmetic, `benchmark` also truncates its average with integer division.

The module defines `time_it` and `benchmark`, which clash with the same names in [time](./time.md) (different meaning and return type); its `run_suite` clashes with the one in [test](./test.md). Do not fetch these modules together in one program.

## Constants

### benchmark_results
A global list that `record_benchmark` and `run_benchmark` append to. Each entry is `{"name": name, "time_us": value}`.

## Functions

### time_it(func)
Calls `func()` once and returns `tick()` after minus `tick()` before (nanoseconds).

### benchmark(func, iterations = 1000)
Calls `time_it(func)` `iterations` times and returns the average. `iterations` of 0 divides by zero.

### benchmark_warmup(func, iterations = 1000, warmup = 100)
Calls `func()` `warmup` times without measuring, then returns `benchmark(func, iterations)`.

### record_benchmark(name, time_us)
Appends `{"name", "time_us"}` to `benchmark_results`.

### run_benchmark(name, func, iterations = 1000)
Benchmarks `func`, records the result, and prints `  <name>: <formatted time>`.

### format_time(us)
Under 1000 gives `"<n> µs"`, under 1000000 gives `"<n/1000> ms"`, otherwise `"<n/1000000> s"`. See the units note above.

### print_results()
Prints a boxed "Benchmark Results" report listing every entry in `benchmark_results`.

### clear_results()
Empties `benchmark_results`.

### compare(name1, func1, name2, func2, iterations = 1000)
Benchmarks both functions, prints a comparison with the speed ratio (`<name> is <n>x faster` or `Both are about the same speed`), and returns `{"time1", "time2", "ratio"}` where `ratio` is `time1 / time2`. If `time2` is 0 the final division hurls a divide-by-zero error (a random Scots prefix, then `Ye numpty! Tryin' tae divide by zero at line 115`). This happens in practice for a very fast function, whose integer average can be 0 (seen: `compare("a", noop, "b", noop, 10)` printed `0 µs` for both, `Both are about the same speed`, then hurled). The ratios are integer divisions, so `2x faster` is truncated.

### estimate_list_memory(lst)
Rough size in bytes: 8 per element, plus string lengths, plus the estimates of nested lists and dicts.

### estimate_dict_memory(obj)
Rough size in bytes: 16 per pair, plus key lengths, string values and nested estimates.

### format_bytes(b)
`"<n> B"` under 1024, `"<n> KB"` under 1048576, otherwise `"<n> MB"` (integer division for integers).

### test_linear(name, generator, func)
Prints a linearity check. For sizes 100, 200, 400 and 800 it calls `generator(size)` to make input, benchmarks `func(data)` for 100 runs, and prints each time. If every consecutive time ratio is between 1.5 and 2.5 it prints `Verdict: Looks like O(n) - linear!`, otherwise `Verdict: Might nae be O(n)`. Returns nothing. A time of 0 for any size makes the ratio calculation fail with a divide-by-zero error.

### generate_numbers(n)
Returns a list of `n` repeatable pseudo-random integers from 0 to 999 (a linear congruential generator with seed 42).

### run_suite(name, benchmarks)
Takes a list of dicts `{"name": ..., "func": ..., "iterations": ...}` (`iterations` optional, default 1000), runs each with `run_benchmark`, and prints a banner and `Suite complete!`.

## Compared with time

[time](./time.md) also has `time_it` and `benchmark`, but those use the millisecond clock `noo()`: `time.time_it` returns `{"result", "ms"}` and `time.benchmark(name, func, iterations)` prints and returns an average in milliseconds. This module uses the nanosecond `tick()` clock, has recording, comparison and suite helpers, and takes the function first. Prefer `benchmark` for micro-benchmarks and comparisons; prefer `time` for a quick `time_an_tell` of a slow operation or for a `Stopwatch`.

## Example

```mdhavers
fetch "stdlib/benchmark"

blether format_bytes(2048)
blether estimate_list_memory([1, 2, "abc", [1, 2]])
blether generate_numbers(5)

ken t = benchmark(|| 1 + 1, 10)
blether t >= 0
run_benchmark("add", || 1 + 1, 10)
blether len(benchmark_results)
```

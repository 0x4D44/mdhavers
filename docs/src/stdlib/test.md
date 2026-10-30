# test

A suite-based testing framework. You create suites with `describe`, add named tests (closures) to them, and run everything with `run_all_tests()`, which prints per-test results, a summary, and returns whether all tests passed. Assertions (`expect_*`) throw on failure; the runner catches the error and records the test as failed. It also has a simple `Mock`/spy and a `TestData` random-data generator.

Import: `fetch "stdlib/test"`

Pure mdhavers; runs on every backend that supports `hae_a_bash`, `mak_siccar`, `noo` and `jammy`. Loading the module prints `Test framework loaded! Ready tae test yer code!`. Output includes emoji and box characters. Helpers starting with `_` (`_test_suites`, `_test_results`, `_verbose`, `_lists_equal`, `_run_single_test`) are internal and not documented.

A failed assertion uses `mak_siccar`, so the error text looks like `Mak siccar failed at line 98! Expected 2 but got 3`. Outside a test runner it ends the program with that error.

Argument order matters: in this module the expected value comes first, `expect_equal(expected, actual)`. The [testing](./testing.md) module uses `(actual, expected)`.

## Functions

### describe(name)
Creates a `TestSuite`, registers it for `run_all_tests`, and returns it.

### test_suite(name)
Alias for `describe`.

### expect_true(condition, message = "Expected condition tae be true")
Fails with `message` unless `condition` is truthy. Returns `aye`.

### expect(condition, message = "Expected condition tae be true")
Alias for `expect_true`.

### expect_false(condition, message = "Expected condition tae be false")
Fails with `message` if `condition` is truthy.

### expect_equal(expected, actual, message = naething)
Fails with `Expected <expected> but got <actual>` (or `message`) when the values differ.

### expect_nae_equal(expected, actual, message = naething)
Fails with `Expected <actual> tae NOT equal <expected>` when the values are equal.

### expect_naething(value, message = "Expected naething (nil)")
Fails with `<message>, but got <value>` unless `value` is `naething`.

### expect_something(value, message = "Expected something, got naething!")
Fails with `message` if `value` is `naething`.

### expect_greater(actual, expected, message = naething)
Fails with `Expected <actual> tae be greater than <expected>` unless `actual > expected`.

### expect_less(actual, expected, message = naething)
Fails with `Expected <actual> tae be less than <expected>` unless `actual < expected`.

### expect_contains(collection, element, message = naething)
Fails with `Expected <collection> tae contain <element>` unless `contains(collection, element)`.

### expect_length(collection, expected_len)
Fails with `Expected length <n> but got <m>`.

### expect_empty(collection)
Fails with `Expected empty collection but got <n> elements`.

### expect_type(value, expected_type)
Compares `whit_kind(value)` with `expected_type` (for example `"integer"`, `"string"`, `"list"`). Fails with `Expected type '<expected>' but got '<actual>'`.

### expect_close(actual, expected, tolerance = 0.0001)
Fails with `Expected <actual> tae be close tae <expected> (tolerance: <t>, diff: <d>)` when the absolute difference exceeds `tolerance`.

### expect_in_range(value, min_val, max_val)
Fails with `Expected <value> tae be between <min> and <max>` (bounds inclusive).

### expect_deep_equal(expected, actual)
For two lists, fails with `Lists differ: <expected> vs <actual>` unless they have the same length and equal elements; for other values it is `expect_equal`.

### expect_error(fn)
Calls `fn()` and fails with `Expected function tae throw an error but it didnae` if it does not hurl.

### run_suite(suite)
Runs one suite: prints its name, each test, and `Suite: <p> passed, <f> failed`. Returns `aye` if nothing failed. Skipped tests are counted in the suite's `passed` figure, but separately in the overall summary. Normally you call `run_all_tests` instead.

### run_all_tests(verbose = aye)
Resets the counters, runs every registered suite, prints a summary (passed, failed, skipped, time in ms, and a list of failures) and returns `aye` when every suite passed. With `verbose` set to `nae` the per-test lines are hidden. Each test runs inside `hae_a_bash`, the suite's before-each hook runs before it and the after-each hook after it.

### reset_tests()
Clears all registered suites and the result counters.

### mock()
Returns a new `Mock`.

### spy(fn)
Returns a `Mock` whose implementation is `fn`, so calls are recorded and forwarded.

### test_data()
Returns a new `TestData`.

## Classes

### TestSuite
`TestSuite(name)`. Fields: `name`, `tests`, `before_each_fn`, `after_each_fn`. All methods return the suite so they chain.

- `add_test(name, test_fn)` adds a test; `test_fn` takes no arguments.
- `skip(name, test_fn)` adds a test that is skipped (reported as skipped, never run).
- `set_before_each(fn)` sets a function run before each test.
- `set_after_each(fn)` sets a function run after each test.

### Mock
`Mock()` records calls. Fields: `calls`, `return_value`, `impl`.

- `returns(value)` sets the fixed return value. Returns the mock.
- `implements(fn)` sets a function that handles calls. Returns the mock.
- `call(arg)` records `arg`, then returns `impl(arg)` if set, otherwise `return_value`. It takes exactly one argument.
- `was_called()` is `aye` if there was at least one call.
- `call_count()` returns the number of calls.
- `last_call()` returns the last argument, or `naething` if there were none.
- `reset()` forgets recorded calls.

### TestData
`TestData()` generates random data using `jammy`. Field `seed` holds the start time and is not used for seeding.

- `random_int(min_val = 0, max_val = 100)` returns a random integer from `min_val` up to but not including `max_val` (it is `jammy(min_val, max_val)`).
- `random_string(length = 10)` returns a string of random lowercase letters. The letter `z` never appears (the index is `jammy(0, 25)`).
- `random_list(length = 5)` returns a list of random integers from 0 to 99.
- `random_bool()` always returns `nae`: it tests `jammy(0, 1) == 1`, and `jammy(0, 1)` can only give 0.

## Compared with testing

Both modules give you `describe`, `expect*` assertions and a way to reset state, but they work differently.

| | test | testing |
| --- | --- | --- |
| Failure | assertion throws; runner catches it | `expect_*` prints a cross and counts, no throw; `assert_*` throws |
| Structure | `describe(name)` returns a suite; you add tests, then call `run_all_tests()` | `describe(name, fn)` and `test(name, fn)` run at once; call `test_summary()` at the end |
| `expect_equal` order | `(expected, actual)` | `(actual, expected)` |
| Extras | hooks, skipped tests, mocks, random data, timings | string, list and type checks, `braw()`/`mince()`, `assert_*` family |

Prefer `test` for structured suites with set-up hooks, mocks and timing. Prefer [testing](./testing.md) for quick scripts where you want assertion lines printed as you go. They both define `describe`, `expect`, `expect_true`, `expect_equal`, `reset_tests` and other names, so fetch only one. [benchmark](./benchmark.md) also defines `run_suite`.

## Example

```mdhavers
fetch "stdlib/test"

ken maths = describe("maths")
maths.add_test("adds", || { expect_equal(2, 1 + 1) })
maths.add_test("lists", || { expect_deep_equal([1, 2], [1, 2]) })
maths.add_test("throws", || { expect_error(|| { hurl "boom" }) })
maths.skip("later", || { expect_true(nae) })

ken ok = run_all_tests()
blether ok
```

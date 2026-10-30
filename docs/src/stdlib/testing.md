# testing

A lightweight, print-as-you-go testing library. Two families of assertions share one pair of global counters (passed and failed): the `expect_*` functions print a tick or cross line and count, never throwing; the `assert_*` functions are silent on success and throw the message on failure. `describe` and `test` group and run tests immediately, and `test_summary()` prints the totals.

Import: `fetch "stdlib/testing"`

Pure mdhavers; runs on every backend that supports `hae_a_bash` and `mak_siccar`. Loading the module prints `Testing module loaded! Time tae mak siccar!`. The state lives in module globals named `__test_passed`, `__test_failed` and `__test_current_suite`; helpers starting with `_` (including `__assert_that`) are internal and not documented.

How failure works: an `expect_*` failure prints a cross line and adds to the failed count but the code carries on. An `assert_*` failure hurls the message (as `Mak siccar failed at line N! <message>`). Inside `test(name, fn)` that error is caught, printed and counted as a failure; outside `test` it ends the program. A test whose body uses only `expect_*` is never marked failed by `test` itself; the counters record the individual failed expectations.

Argument order: `expect_equal(actual, expected)` and `assert_eq(actual, expected, msg)` take the actual value first. The [test](./test.md) module uses `(expected, actual)`.

## Functions

### expect(condition, message = "Expectation failed")
Prints `  ✓ <message>` and counts a pass when `condition` is truthy, otherwise `  ✗ <message>` and counts a failure. Returns the condition. All other `expect_*` functions are built on it.

### expect_equal(actual, expected, message = "")
Pass/fail on `actual == expected`. The default message is `Expected <expected>, got <actual>`. A failure also prints `Expected:` and `Actual:` lines. Returns `aye` or `nae`.

### expect_not_equal(actual, unexpected, message = "")
Passes when the values differ. Default message `Expected NOT <unexpected>`.

### expect_true(value, message = "Expected true")
Passes only when `value == aye`.

### expect_false(value, message = "Expected false")
Passes only when `value == nae`.

### expect_naething(value, message = "Expected naething")
Passes when `value` is `naething`.

### expect_somethin(value, message = "Expected something (not naething)")
Passes when `value` is not `naething`.

### expect_greater(actual, threshold, message = "")
Passes on `actual > threshold`. Default message `Expected <actual> > <threshold>`.

### expect_less(actual, threshold, message = "")
Passes on `actual < threshold`.

### expect_between(actual, wee_end, muckle_end, message = "")
Passes when `wee_end <= actual <= muckle_end`.

### expect_near(actual, expected, tolerance = 0.0001, message = "")
Passes when the absolute difference is at most `tolerance`.

### expect_length(list, expected_len, message = "")
Passes when `len(list) == expected_len`.

### expect_toom(list, message = "Expected empty list")
Passes when the list is empty (`toom` is Scots for empty).

### expect_no_toom(list, message = "Expected non-empty list")
Passes when the list has elements.

### expect_contains(list, element, message = "")
Passes when `contains(list, element)`.

### expect_no_contains(list, element, message = "")
Passes when the list does not contain `element`.

### expect_lists_equal(actual, expected, message = "")
Passes when the lists have the same length and equal elements in order. On failure prints `Length mismatch: <a> vs <b>` or `Differs at index <i>: <x> vs <y>`.

### expect_starts_wi(s, prefix, message = "")
Passes when string `s` starts with `prefix`.

### expect_ends_wi(s, suffix, message = "")
Passes when string `s` ends with `suffix`.

### expect_str_contains(s, needle, message = "")
Passes when `contains(s, needle)`.

### expect_type(value, expected_type, message = "")
Compares `whit_kind(value)` to `expected_type`. Type names are those of `whit_kind`, for example `"integer"`, not `"int"`.

### describe(suite_name, tests_fn = naething)
Records the suite name and prints `═══ <suite_name> ═══`. If `tests_fn` is given it is called straight away and a blank line follows; without it, the tests that follow belong to the suite by convention only.

### test(test_name, test_fn)
Prints `  Testing: <test_name>` and calls `test_fn()` inside `hae_a_bash`. If it hurls (for example from an `assert_*`), prints `  ✗ <test_name>: <error>` and counts a failure.

### test_summary()
Prints totals (total, passed, failed) and a cheery or glum message. Returns `aye` when nothing failed. Does not reset the counters.

### reset_tests()
Sets passed and failed back to 0 and clears the current suite name.

### test_reset()
Alias for `reset_tests()`.

### braw(message = "That's braw!")
Prints a pass line and counts a pass. Returns `aye`.

### mince(message = "That's pure mince!")
Prints a fail line and counts a failure. Returns `nae`.

### skip(reason = "Test skipped")
Prints `  ⊘ SKIP: <reason>`. Not counted.

### todo(what = "Test not implemented yet")
Prints `  📝 TODO: <what>`. Not counted.

### assert_eq(actual, expected, msg)
Counts a pass if equal. Otherwise prints `Expected: <expected>` and `Actual: <actual>` and hurls `msg`.

### assert_ne(actual, expected, msg)
Hurls `msg` if the values are equal.

### assert_true(val, msg)
Hurls `msg` unless `val == aye`.

### assert_false(val, msg)
Hurls `msg` unless `val == nae`.

### assert_nil(val, msg)
Hurls `msg` unless `val` is `naething`.

### assert_not_nil(val, msg)
Hurls `msg` if `val` is `naething`.

### assert_contains(list, item, msg)
Hurls `msg` unless the list contains `item`.

### assert_not_contains(list, item, msg)
Hurls `msg` if the list contains `item`.

### assert_empty(list, msg)
Hurls `msg` unless the list is empty.

### assert_not_empty(list, msg)
Hurls `msg` if the list is empty.

### assert_in_range(val, min_val, max_val, msg)
Hurls `msg` unless `min_val <= val <= max_val`.

### assert_gt(actual, expected, msg)
Hurls `msg` unless `actual > expected`.

### assert_gte(actual, expected, msg)
Hurls `msg` unless `actual >= expected`.

### assert_lt(actual, expected, msg)
Hurls `msg` unless `actual < expected`.

### assert_lte(actual, expected, msg)
Hurls `msg` unless `actual <= expected`.

### assert_starts_with(s, prefix, msg)
Hurls `msg` unless `s` starts with `prefix` (uses `scran`).

### assert_ends_with(s, suffix, msg)
Hurls `msg` unless `s` ends with `suffix`.

### assert_approx(actual, expected, epsilon, msg)
Hurls `msg` unless `abs(actual - expected) < epsilon`.

### assert_list_eq(actual, expected, msg)
Hurls `msg` unless the lists are equal. Before hurling it prints `Length mismatch: <a> vs <b>` or `Mismatch at index <i>: <x> vs <y>`.

### assert_throws(fn, msg)
Calls `fn()`; hurls `msg` if it does not hurl.

All the `assert_*` functions take their message as a required last argument and hurl it, wrapped as `Mak siccar failed at line N! <msg>`.

## Compared with test

See the table in [test](./test.md#compared-with-testing). In short, `testing` is the simpler, immediate-mode library: no suite objects, no hooks, no mocks, `expect_*` never throws, and `expect_equal` takes `(actual, expected)`. [test](./test.md) is the structured runner. Both define `describe`, `expect`, `expect_true`, `expect_false`, `expect_equal`, `expect_naething`, `expect_greater`, `expect_less`, `expect_length`, `expect_contains`, `expect_type` and `reset_tests` with different behaviour, so fetch only one of them. Prefer `testing` for small scripts and examples, and `test` for larger suites that need set-up hooks or mocks.

## Example

```mdhavers
fetch "stdlib/testing"

describe("maths", || {
    test("addition", || {
        expect_equal(1 + 1, 2, "one plus one is two")
        assert_eq(2 * 3, 6, "two times three is six")
    })
    test("lists", || {
        expect_lists_equal([1, 2, 3], [1, 2, 3], "lists match")
        assert_throws(|| { hurl "boom" }, "should hurl")
    })
})

blether test_summary()
```

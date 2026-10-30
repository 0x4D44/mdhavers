# promise

Promise-like and monad-like helpers: `Promise` with success, error and finally handlers; `Result` (ok or error); `Option` (some or none); `Lazy` evaluation; a sequential `TaskQueue`; and a `Pipeline` of functions.

Import: `fetch "stdlib/promise"`

Pure mdhavers; it runs on every backend. mdhavers is single-threaded, so these are synchronous: a promise settles the moment `resolve` or `reject` is called and handlers run right then. Nothing is scheduled for later. Loading the module prints `Promise module loaded! Ready tae make some promises!`. Helpers starting with `_` are internal and not documented (this module has none). A brace-less lambda (`|x| x + 1`) is a single expression; use a `{ }` block lambda or a named `dae` if you need statements.

## Constants

`PENDING` (`"pending"`), `FULFILLED` (`"fulfilled"`), `REJECTED` (`"rejected"`) - the values of `Promise.state`.

## Functions

### resolved(value)
A promise already fulfilled with `value`.

### rejected(error)
A promise already rejected with `error`.

### ok_result(value)
A `Result` holding `value` with `is_ok` set to `aye`.

### err_result(error)
A `Result` holding `error` with `is_ok` set to `nae`.

### some_option(value)
An `Option` that has `value`, even if `value` is `naething`.

### none_option()
An empty `Option`.

## Classes

### Promise
Constructor: `Promise(executor = naething)`. If an executor is given it runs immediately as `executor(resolve_fn, reject_fn)`; if it hurls, the promise is rejected with the error (for `hurl "oops"` the stored error is the text `Hurled at line 19: oops`, with the line number of the `hurl`). Fields: `state`, `value`, `error`, and the handler lists.
- `resolve(value)` - if still pending, fulfils, then runs the success handlers and the finally handlers. Ignored once settled. Returns `masel`.
- `reject(error)` - if still pending, rejects, then runs the catch handlers and the finally handlers. Returns `masel`.
- `on_success(handler)` - `handler(value)`; runs at once if already fulfilled, queued if pending, ignored if rejected. Returns `masel`. (There is no method called `then`.)
- `catch(handler)` - `handler(error)`; same rules for the rejected case.
- `finally(handler)` - `handler()` once settled either way; at once if already settled.
- `is_pending()`, `is_fulfilled()`, `is_rejected()`.
- `await()` - the value if fulfilled, the error if rejected, `naething` if pending. It never waits and does not distinguish a rejection from a fulfilment; check `is_rejected()` first.

### Result
Either a value or an error. Constructor: `Result(value = naething, error = naething)`; `is_ok` is `aye` when `error` is `naething`. Fields: `value`, `error`, `is_ok`.
- `ok(value)` and `err(error)` - set the result in place and return `masel` (they are mutators, not static constructors; prefer `ok_result` and `err_result`).
- `map(fn)` - new ok result of `fn(value)`, or a new err result with the same error.
- `map_err(fn)` - new err result of `fn(error)`, or a new ok result with the same value.
- `unwrap_or(default)`, `unwrap_or_else(fn)` (calls `fn(error)` when an error).
- `is_success()`, `is_failure()`.

### Option
A value that may be absent. Constructor: `Option(value = naething)`; `has_value` is `value != naething`, so `Option(0)` is some. Fields: `value`, `has_value`.
- `some(value)` and `none()` - mutate in place and return `masel`; prefer `some_option` and `none_option`.
- `is_some()`, `is_none()`.
- `map(fn)` - new option of `fn(value)`, or none.
- `unwrap_or(default)`, `unwrap_or_else(fn)` (calls `fn()`).
- `filter(predicate)` - the same option if present and the predicate holds, else none.

### Lazy
Deferred, memoised computation. Constructor: `Lazy(thunk)`; `thunk` takes no arguments. Fields: `thunk`, `evaluated`, `value`.
- `force()` - runs the thunk once, caches and returns the value.
- `is_evaluated()`.
- `map(fn)` - a new `Lazy` of `fn(force())`; nothing runs until it is forced.

### TaskQueue
Runs zero-argument functions in order. Constructor: `TaskQueue()`. Fields: `tasks`, `results`.
- `add(task)` - returns `masel`. `run()` - calls every task, stores and returns the list of results. `get_results()`, `clear()`, `count()` (number of tasks).
- A task that hurls aborts `run()`; the error propagates.

### Pipeline
Constructor: `Pipeline()`. Field: `steps`.
- `pipe(fn)` - append a step, returns `masel`. `execute(initial_value)` - feeds the value through each step. `length()` - number of steps.

## Example

```mdhavers
fetch "stdlib/promise"

dae show(v) {
    blether f"got {v}"
}

ken p = Promise()
p.on_success(show)
p.resolve(42)

blether ok_result(5).map(|x| x * 2).unwrap_or(0)
blether none_option().unwrap_or("default")

ken pl = Pipeline()
pl.pipe(|x| x + 1).pipe(|x| x * 2)
blether pl.execute(3)
```

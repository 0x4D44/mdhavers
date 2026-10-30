# MDH-BUG-FLU-00220 — stopwatch() is a placeholder: never calls or times its function

- **State:** Open
- **Priority:** Could
- **Severity:** Low
- **Area:** interpreter
- **Raised:** 2026-10-30T07:01:23Z
- **Discovery source:** Agent
- **Owner:** -
- **Owner role:** -
- **Owner run:** -
- **Owner host:** -
- **Owner branch:** -
- **Owner base:** -
- **Owner fingerprint:** -
- **Owner since:** -
- **Owner until:** -
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-30T07:01:23Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

**Symptom:** `stopwatch(f)` never calls `f` and never times anything. In the interpreter it returns a hint string instead of the documented `[result, time_ms]`.

**Repro (interpreter, 0.3.137):**
```
dae work() { gie 42 }
blether stopwatch(work)
```
Prints `Use 'noo()' before and after callin' 'work' tae time it!`. Expected (per the comment at src/interpreter.rs:8668, "time a function call and return [result, time_ms]") a list like `[42, <ms>]`.

**Where:** src/interpreter.rs:8668-8682. The closure is a NativeFunction without interpreter access, and its own comment calls it a placeholder.

Found by the docs worker while documenting every built-in (2026-10-30). The docs currently describe the placeholder behaviour as it is.

Evidence fingerprint: `manual:v1:stopwatch-is-a-placeholder-never-calls-or-times-its-function`


## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00002 — Self-referential dict or list crashes the interpreter on assignment even when never printed

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:06Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T014808Z-2431ecf4
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00002-run-fix-20261023T014808Z-2431ecf4
- **Owner base:** 1c1ad1c4371ae9f9bc88ffcc726f209d9fb256a3
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:48:08Z
- **Owner until:** 2026-10-23T05:48:08Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:06Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: `ken d = {}` then `d["me"] = d` aborts with stack overflow at that statement, because the statement value is formatted for trace output even with tracing off (interpreter.rs ~10659 trace() takes an eagerly-formatted &str) and Value Display (value.rs:226) recurses without cycle detection. Same for `shove(a, a)` then `ken b = a`, and structural list == recursion (value.rs:272).

## Fix

<unfixed — raised only>

## Notes

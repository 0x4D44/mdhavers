# MDH-BUG-FLU-00002 — Self-referential dict or list crashes the interpreter on assignment even when never printed

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:06Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-b3955bb0
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00002-run-verify-20261026T214519Z-b3955bb0
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:857ef57cb0636fd961065b8caa3483929ee55dd621275a6e7cfdc5908bc79bf7
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:06Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:48:14Z, deltic:auto role=fix run=fix-20261023T014808Z-2431ecf4 branch=task/bug-MDH-BUG-FLU-00002-run-fix-20261023T014808Z-2431ecf4 code=d84823a gate=manual)

## Observation

Confirmed: `ken d = {}` then `d["me"] = d` aborts with stack overflow at that statement, because the statement value is formatted for trace output even with tracing off (interpreter.rs ~10659 trace() takes an eagerly-formatted &str) and Value Display (value.rs:226) recurses without cycle detection. Same for `shove(a, a)` then `ken b = a`, and structural list == recursion (value.rs:272).

## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00002 — Self-referential dict or list crashes the interpreter on assignment even when never printed

- **State:** Closed
- **Priority:** Should
- **Severity:** High
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:06Z
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
- **State history:** Open (2026-10-22T23:24:06Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:48:14Z, deltic:auto role=fix run=fix-20261023T014808Z-2431ecf4 branch=task/bug-MDH-BUG-FLU-00002-run-fix-20261023T014808Z-2431ecf4 code=d84823a gate=manual) -> Closed (2026-10-26T22:19:14Z, deltic:manual role=verify run=verify-20261026T214519Z-b3955bb0 verified_fix_run=fix-20261023T014808Z-2431ecf4 verdict=close)

## Observation

Confirmed: `ken d = {}` then `d["me"] = d` aborts with stack overflow at that statement, because the statement value is formatted for trace output even with tracing off (interpreter.rs ~10659 trace() takes an eagerly-formatted &str) and Value Display (value.rs:226) recurses without cycle detection. Same for `shove(a, a)` then `ken b = a`, and structural list == recursion (value.rs:272).

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T22:19:14Z, deltic:manual run=verify-20261026T214519Z-b3955bb0 verified_fix_run=fix-20261023T014808Z-2431ecf4 verdict=close)

Verifier note: Manual verify: [verifier gpt-6.1-sol (Codex), recorded by lead] d84823a; self_referential_dict_survives_assignment_and_prints selected/pass; remove cycle detection, rerun with 16MiB test stack -> output assertion fails (nested dict vs {"me": {...}}). Original assignment/print survives; source restored, diff empty.

## Notes

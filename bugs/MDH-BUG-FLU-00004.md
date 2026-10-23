# MDH-BUG-FLU-00004 — Unchecked integer maths in natives panics or silently wraps (abs, pooer, gcd, lcm, jammy)

- **State:** Fixed
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
- **State history:** Open (2026-10-22T23:24:06Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:48:36Z, deltic:auto role=fix run=fix-20261023T014830Z-f8bfe59f branch=task/bug-MDH-BUG-FLU-00004-run-fix-20261023T014830Z-f8bfe59f code=d84823a gate=manual)

## Observation

Confirmed: abs(i64::MIN) panics "attempt to negate with overflow" in debug (wraps in release); gcd(i64::MIN, -1) panics "remainder with overflow" in release too (interpreter.rs ~9103/9121); pooer(2,64) -> 0, pooer exponent cast `as u32` truncates; lcm and jammy max-min wrap. Should raise IntegerOverflow.

## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00004 — Unchecked integer maths in natives panics or silently wraps (abs, pooer, gcd, lcm, jammy)

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
- **State history:** Open (2026-10-22T23:24:06Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:48:36Z, deltic:auto role=fix run=fix-20261023T014830Z-f8bfe59f branch=task/bug-MDH-BUG-FLU-00004-run-fix-20261023T014830Z-f8bfe59f code=d84823a gate=manual) -> Closed (2026-10-26T22:19:14Z, deltic:manual role=verify run=verify-20261026T214519Z-4b80891b verified_fix_run=fix-20261023T014830Z-f8bfe59f verdict=close)

## Observation

Confirmed: abs(i64::MIN) panics "attempt to negate with overflow" in debug (wraps in release); gcd(i64::MIN, -1) panics "remainder with overflow" in release too (interpreter.rs ~9103/9121); pooer(2,64) -> 0, pooer exponent cast `as u32` truncates; lcm and jammy max-min wrap. Should raise IntegerOverflow.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T22:19:14Z, deltic:manual run=verify-20261026T214519Z-4b80891b verified_fix_run=fix-20261023T014830Z-f8bfe59f verdict=close)

Verifier note: Manual verify: [verifier gpt-6.1-sol (Codex), recorded by lead] d84823a; native_integer_overflow_is_an_error_not_a_panic_or_wrap selected/pass; replace checked_pow with wrapping_pow -> assert_overflow_caught fails: pooer(2,64) gave 0. Baseline abs(MIN), gcd(MIN,-1), lcm and extreme random-range tests pass with catchable errors/exact values. Restored; diff empty.

## Notes

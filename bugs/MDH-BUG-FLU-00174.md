# MDH-BUG-FLU-00174 — Native RC leaks in string destructuring, items, roar, log_span, module alias, grup_up

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-26T05:04:17Z
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
- **State history:** Open (2026-10-26T05:04:17Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T08:27:50Z, deltic:auto role=fix run=fix-20261026T062548Z-7823bb9e branch=task/bug-MDH-BUG-FLU-00174-run-fix-20261026T062548Z-7823bb9e code=349da95 gate=manual) -> Closed (2026-10-26T21:44:08Z, deltic:manual role=verify run=verify-20261026T194306Z-646c7cbf verified_fix_run=fix-20261026T062548Z-7823bb9e verdict=close)

## Observation

Found by lane H1 2026-10-26 (MDH_RC=1 builds, MDH_RC_REPORT=1 total != 0 at exit; output correct): `ken [c1, c2] = "AB"` leaks 3 (list destructuring is clean); `ken p = items({"a": 1})` leaks 2 lists; `blether roar("hello")` and `shout("hi")` leak 1 string; `ken sp = log_span("s")` leaks 6; `fetch "lib/strings" tae str` leaks 1 dict; `grup_up([1,2,3], |x| x % 2)` leaks 3 and `pairt_by([1,2,3], |x| x > 1)` leaks 2. Unreduced corpus leaks: highland_adventure 268, blether_lisp 42807, config_differ 13 (store_demo 16 is likely cycles). Not covered here (accepted by Arthur, round 2): reference cycles and the error message held by a caught or uncaught runtime error. Fix each owner in the ownership table / runtime with a total=0 test.

Evidence fingerprint: `manual:v1:native-rc-leaks-in-string-destructuring-items-r-f41eab5caa216d33`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T21:44:08Z, deltic:manual run=verify-20261026T194306Z-646c7cbf verified_fix_run=fix-20261026T062548Z-7823bb9e verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] 349da95; all 11 llvm_rc_leaks tests pass on trunk. Reversing ownership/counting hunks made 9 leak assertions fail: destructuring total=3, items=2 lists, roar=1 string, log_span total=6, and grup_up/pairt_by/module/early-return leaks.

## Notes

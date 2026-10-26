# MDH-BUG-FLU-00174 — Native RC leaks in string destructuring, items, roar, log_span, module alias, grup_up

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-26T05:04:17Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T194306Z-646c7cbf
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00174-run-verify-20261026T194306Z-646c7cbf
- **Owner base:** 36750bc8afd262a99873992d919d9ed250946aef
- **Owner fingerprint:** sha256:7d29de7ab4f61e302adcfbd37429a55ca45bd05ecd4ddeac746217a97a64bb95
- **Owner since:** 2026-10-26T19:43:06Z
- **Owner until:** 2026-10-26T23:43:06Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T05:04:17Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T08:27:50Z, deltic:auto role=fix run=fix-20261026T062548Z-7823bb9e branch=task/bug-MDH-BUG-FLU-00174-run-fix-20261026T062548Z-7823bb9e code=349da95 gate=manual)

## Observation

Found by lane H1 2026-10-26 (MDH_RC=1 builds, MDH_RC_REPORT=1 total != 0 at exit; output correct): `ken [c1, c2] = "AB"` leaks 3 (list destructuring is clean); `ken p = items({"a": 1})` leaks 2 lists; `blether roar("hello")` and `shout("hi")` leak 1 string; `ken sp = log_span("s")` leaks 6; `fetch "lib/strings" tae str` leaks 1 dict; `grup_up([1,2,3], |x| x % 2)` leaks 3 and `pairt_by([1,2,3], |x| x > 1)` leaks 2. Unreduced corpus leaks: highland_adventure 268, blether_lisp 42807, config_differ 13 (store_demo 16 is likely cycles). Not covered here (accepted by Arthur, round 2): reference cycles and the error message held by a caught or uncaught runtime error. Fix each owner in the ownership table / runtime with a total=0 test.

Evidence fingerprint: `manual:v1:native-rc-leaks-in-string-destructuring-items-r-f41eab5caa216d33`


## Fix

<unfixed — raised only>

## Notes

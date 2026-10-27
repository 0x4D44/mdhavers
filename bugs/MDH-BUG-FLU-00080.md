# MDH-BUG-FLU-00080 — Anonymous dae expressions and keyword-named lambda params fail to parse in shipped files

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-23T02:06:58Z
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
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:16:04Z, deltic:auto role=fix run=fix-20261023T031558Z-b83feee4 branch=task/bug-MDH-BUG-FLU-00080-run-fix-20261023T031558Z-b83feee4 code=e5fbb29 gate=manual) -> Closed (2026-10-27T01:51:48Z, deltic:manual role=verify run=verify-20261026T233955Z-1788b04e verified_fix_run=fix-20261023T031558Z-b83feee4 verdict=close)

## Observation

Found by the parser lane (pre-existing): benchmarks/stress/stability_test.braw:25 uses `dae(acc, x) {...}` as an expression; stdlib/tri.braw:23 uses `|thing|` where thing is a keyword.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:51:48Z, deltic:manual run=verify-20261026T233955Z-1788b04e verified_fix_run=fix-20261023T031558Z-b83feee4 verdict=close)

Verifier note: Manual verify: [lc luna] e5fbb29: tri_stub_and_stability_benchmark_parse passed; restoring `thing` failed on line 23, and restoring `dae(...)` callbacks failed on line 25. Both mutations reproduced the recorded parse errors; source diff restored clean.

## Notes

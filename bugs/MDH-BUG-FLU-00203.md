# MDH-BUG-FLU-00203 — JS transcendental parity test passes silently when node is missing

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** tests
- **Raised:** 2026-10-27T05:23:56Z
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
- **State history:** Open (2026-10-27T05:23:56Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-27T06:12:28Z, deltic:auto role=fix run=fix-20261027T055918Z-a13d06e8 branch=task/bug-MDH-BUG-FLU-00203-run-fix-20261027T055918Z-a13d06e8 code=5acc548 gate=manual) -> Closed (2026-10-27T06:19:36Z, deltic:manual role=verify run=verify-20261027T061519Z-c04c7c13 verified_fix_run=fix-20261027T055918Z-a13d06e8 verdict=close)

## Observation

tests/backend_parity.rs `parity_js_transcendentals_agree_when_rounded_mdh_bug_flu_00172` (895148e) returns early, green, when `node` is not on PATH, so on a box without Node it passes having checked nothing. Found by the independent reviewer closing MDH-BUG-FLU-00172 (2026-10-27). The same reviewer noted the 172 Observation also asked that no golden print full-precision transcendentals; that is unconfirmed.

Expected: a missing Node either fails the test or reports a visible skip (the repo's convention for missing prerequisites; check how other JS tests handle it), and the goldens are checked for full-precision transcendental output.

Evidence fingerprint: `manual:v1:js-transcendental-parity-test-passes-silently-w-d08520229a75bdfd`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T06:19:36Z, deltic:manual run=verify-20261027T061519Z-c04c7c13 verified_fix_run=fix-20261027T055918Z-a13d06e8 verdict=close)

Verifier note: Manual verify: [li luna] 5acc548; parity_js_transcendentals_agree_when_rounded_mdh_bug_flu_00172 passed 1/1 with Node; with PATH=/usr/bi[PATH] and --nocapture it selected 1/1 and printed SKIPPED: node is no' on PATH; checked nothing; other JS tests use SKIPPED; math goldens emit simple values or integer/image output.

## Notes

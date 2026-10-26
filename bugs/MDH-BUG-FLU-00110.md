# MDH-BUG-FLU-00110 — JS: sieve, aw, ony and hunt use JS truthiness instead of mdhavers truthiness

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-24T02:39:01Z
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
- **State history:** Open (2026-10-24T02:39:01Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T02:53:33Z, deltic:auto role=fix run=fix-20261024T025326Z-0ba68638 branch=task/bug-MDH-BUG-FLU-00110-run-fix-20261024T025326Z-0ba68638 code=9a17225 gate=manual) -> Closed (2026-10-26T23:39:42Z, deltic:manual role=verify run=verify-20261026T225713Z-05e3d831 verified_fix_run=fix-20261024T025326Z-0ba68638 verdict=close)

## Observation

Found by the lead's review 2026-10-24 (read). src/compiler.rs ~1004-1009 map them to filter/every/some/find directly. Repro: blether len(sieve([{}, {"a": 1}, []], |d| d)) -> interpreter 1, JS 3. Wrap predicates in __havers.truthy.

Evidence fingerprint: `manual:v1:js-sieve-aw-ony-and-hunt-use-js-truthiness-inst-821bdd341f927bfb`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:39:42Z, deltic:manual run=verify-20261026T225713Z-05e3d831 verified_fix_run=fix-20261024T025326Z-0ba68638 verdict=close)

Verifier note: Manual verify: [lb luna] 9a17225 (implementation ba51a88); parity_js_predicates_use_mdhavers_truthiness_mdh_bug_flu_00110 passed; reverting `sieve` to JS filter truthiness failed: expected length 1, got 3 (empty dict/list kept).

## Notes

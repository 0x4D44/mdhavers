# MDH-BUG-FLU-00110 — JS: sieve, aw, ony and hunt use JS truthiness instead of mdhavers truthiness

- **State:** Open
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
- **State history:** Open (2026-10-24T02:39:01Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by the lead's review 2026-10-24 (read). src/compiler.rs ~1004-1009 map them to filter/every/some/find directly. Repro: blether len(sieve([{}, {"a": 1}, []], |d| d)) -> interpreter 1, JS 3. Wrap predicates in __havers.truthy.

Evidence fingerprint: `manual:v1:js-sieve-aw-ony-and-hunt-use-js-truthiness-inst-821bdd341f927bfb`


## Fix

<unfixed — raised only>

## Notes

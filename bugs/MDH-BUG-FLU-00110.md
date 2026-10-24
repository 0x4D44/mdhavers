# MDH-BUG-FLU-00110 — JS: sieve, aw, ony and hunt use JS truthiness instead of mdhavers truthiness

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-24T02:39:01Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T025326Z-0ba68638
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00110-run-fix-20261024T025326Z-0ba68638
- **Owner base:** 08dbcf4245071b0fc8b60f2d7443b3a96a78140a
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T02:53:26Z
- **Owner until:** 2026-10-24T06:53:26Z
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

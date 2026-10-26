# MDH-BUG-FLU-00110 — JS: sieve, aw, ony and hunt use JS truthiness instead of mdhavers truthiness

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-24T02:39:01Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T225713Z-05e3d831
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00110-run-verify-20261026T225713Z-05e3d831
- **Owner base:** 6cca1b9724e089b14af0018739faf304b5548a5a
- **Owner fingerprint:** sha256:f0b6ea32c6079855671c2e83ea883872bb5a3f05a509a128e109080cf734be1f
- **Owner since:** 2026-10-26T22:57:13Z
- **Owner until:** 2026-10-27T02:57:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T02:39:01Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T02:53:33Z, deltic:auto role=fix run=fix-20261024T025326Z-0ba68638 branch=task/bug-MDH-BUG-FLU-00110-run-fix-20261024T025326Z-0ba68638 code=9a17225 gate=manual)

## Observation

Found by the lead's review 2026-10-24 (read). src/compiler.rs ~1004-1009 map them to filter/every/some/find directly. Repro: blether len(sieve([{}, {"a": 1}, []], |d| d)) -> interpreter 1, JS 3. Wrap predicates in __havers.truthy.

Evidence fingerprint: `manual:v1:js-sieve-aw-ony-and-hunt-use-js-truthiness-inst-821bdd341f927bfb`


## Fix

<unfixed — raised only>

## Notes

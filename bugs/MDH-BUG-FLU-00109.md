# MDH-BUG-FLU-00109 — JS: integer dict keys come back as strings and now fail BigInt-strict list indexing

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-24T02:38:57Z
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
- **State history:** Open (2026-10-24T02:38:57Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T02:53:20Z, deltic:auto role=fix run=fix-20261024T025313Z-c6c0414b branch=task/bug-MDH-BUG-FLU-00109-run-fix-20261024T025313Z-c6c0414b code=9a17225 gate=manual) -> Closed (2026-10-26T23:39:42Z, deltic:manual role=verify run=verify-20261026T225713Z-ab9d4707 verified_fix_run=fix-20261024T025313Z-c6c0414b verdict=close)

## Observation

Found by the lead's review of 7fbcda1 2026-10-24 (read). keys() is Object.keys (src/compiler.rs ~826), so integer keys become strings; runtime idx/setIdx reject non-BigInt indexes. Repro: ken d = {0: "a"}  ken xs = [10]  fer k in keys(d) { blether xs[k] } -> interpreter 10, JS "Cannae index a list wi' a string". whit_kind(k) and k + 1 also diverge. Dicts need typed keys in JS (e.g. Map with a key encoding).

Evidence fingerprint: `manual:v1:js-integer-dict-keys-come-back-as-strings-and-n-dd2cb88ce827618b`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:39:42Z, deltic:manual run=verify-20261026T225713Z-ab9d4707 verified_fix_run=fix-20261024T025313Z-c6c0414b verdict=close)

Verifier note: Manual verify: [lb luna] 9a17225 (implementation ba51a88); parity_js_typed_dict_keys_mdh_bug_flu_00109 passed; disabling `undkey` made integer `keys(d)` become strings and the test failed: `Cannae index a list wi\x27 a string`.

## Notes

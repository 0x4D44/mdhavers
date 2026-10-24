# MDH-BUG-FLU-00109 — JS: integer dict keys come back as strings and now fail BigInt-strict list indexing

- **State:** Open
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
- **State history:** Open (2026-10-24T02:38:57Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by the lead's review of 7fbcda1 2026-10-24 (read). keys() is Object.keys (src/compiler.rs ~826), so integer keys become strings; runtime idx/setIdx reject non-BigInt indexes. Repro: ken d = {0: "a"}  ken xs = [10]  fer k in keys(d) { blether xs[k] } -> interpreter 10, JS "Cannae index a list wi' a string". whit_kind(k) and k + 1 also diverge. Dicts need typed keys in JS (e.g. Map with a key encoding).

Evidence fingerprint: `manual:v1:js-integer-dict-keys-come-back-as-strings-and-n-dd2cb88ce827618b`


## Fix

<unfixed — raised only>

## Notes

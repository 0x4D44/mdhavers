# MDH-BUG-FLU-00109 — JS: integer dict keys come back as strings and now fail BigInt-strict list indexing

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-24T02:38:57Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T225713Z-ab9d4707
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00109-run-verify-20261026T225713Z-ab9d4707
- **Owner base:** 6cca1b9724e089b14af0018739faf304b5548a5a
- **Owner fingerprint:** sha256:343b9848e3b86ddf3381779fda7010163baffdda873b0849656bd04f0d2099d1
- **Owner since:** 2026-10-26T22:57:13Z
- **Owner until:** 2026-10-27T02:57:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T02:38:57Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T02:53:20Z, deltic:auto role=fix run=fix-20261024T025313Z-c6c0414b branch=task/bug-MDH-BUG-FLU-00109-run-fix-20261024T025313Z-c6c0414b code=9a17225 gate=manual)

## Observation

Found by the lead's review of 7fbcda1 2026-10-24 (read). keys() is Object.keys (src/compiler.rs ~826), so integer keys become strings; runtime idx/setIdx reject non-BigInt indexes. Repro: ken d = {0: "a"}  ken xs = [10]  fer k in keys(d) { blether xs[k] } -> interpreter 10, JS "Cannae index a list wi' a string". whit_kind(k) and k + 1 also diverge. Dicts need typed keys in JS (e.g. Map with a key encoding).

Evidence fingerprint: `manual:v1:js-integer-dict-keys-come-back-as-strings-and-n-dd2cb88ce827618b`


## Fix

<unfixed — raised only>

## Notes

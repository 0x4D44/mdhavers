# MDH-BUG-FLU-00102 — Native: an empty list, dict or string read straight from an index is truthy in a condition

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-24T02:09:47Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T182021Z-29e8d75b
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00102-run-verify-20261026T182021Z-29e8d75b
- **Owner base:** 2f352f49ef4c71b45a0497bf81a04779c622faad
- **Owner fingerprint:** sha256:f682744ad29e40fd54b247ac8daea29f8f94046054c6487560267782b069da69
- **Owner since:** 2026-10-26T18:20:21Z
- **Owner until:** 2026-10-26T22:20:21Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T02:09:47Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T02:38:32Z, deltic:auto role=fix run=fix-20261024T023826Z-cf7665f9 branch=task/bug-MDH-BUG-FLU-00102-run-fix-20261024T023826Z-cf7665f9 code=e92c292 gate=manual)

## Observation

Found by lane X1 (showcases) 2026-10-24. src/llvm/codegen.rs compile_condition_direct, Expr::Index arm (~5260-5355): list fast path and fallback test data != 0 instead of is_truthy. Repro: ken d = {"l": [], "m": {}, "s": ""}  gin d["l"] { blether "wrang" } prints wrang natively; interpreter/JS print nothing. Same for d["m"], d["s"], xs[0] with xs = [[]]. ken h = d["l"]; gin h is correct. Golden tests/golden/integration/showcase_bothy_adventure.braw is SKIP_NATIVE on this.

Evidence fingerprint: `manual:v1:native-an-empty-list-dict-or-string-read-straig-97cca045b944f5f2`


## Fix

<unfixed — raised only>

## Notes

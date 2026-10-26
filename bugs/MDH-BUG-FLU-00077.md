# MDH-BUG-FLU-00077 — JS backend sort uses its own ordering, diverging from the interpreter on mixed types

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** js
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
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:45:08Z, deltic:auto role=fix run=fix-20261023T034502Z-95b64d22 branch=task/bug-MDH-BUG-FLU-00077-run-fix-20261023T034502Z-95b64d22 code=54979e9 gate=manual) -> Closed (2026-10-26T23:39:42Z, deltic:manual role=verify run=verify-20261026T225713Z-ad7a735d verified_fix_run=fix-20261023T034502Z-95b64d22 verdict=close)

## Observation

Found by the interpreter lane: src/compiler.rs ~258 sort helper does not follow the interpreter's new total order (numbers, then strings, then others).

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:39:42Z, deltic:manual run=verify-20261026T225713Z-ad7a735d verified_fix_run=fix-20261023T034502Z-95b64d22 verdict=close)

Verifier note: Manual verify: [lb luna] 54979e9; parity_js_sort_total_order_mdh_bug_flu_00077 passed; disabling JS type ranks failed its parity assertion: expected [2, 7, 10, B, a, b, naething, aye], printed [10, b, 2, B, a, naething, aye, 7].

## Notes

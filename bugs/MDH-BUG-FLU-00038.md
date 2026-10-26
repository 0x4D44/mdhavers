# MDH-BUG-FLU-00038 — JS compiler splices import paths into require() unescaped

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-22T23:24:10Z
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
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:16:41Z, deltic:auto role=fix run=fix-20261023T021635Z-5347512d branch=task/bug-MDH-BUG-FLU-00038-run-fix-20261023T021635Z-5347512d code=3e24904 gate=manual) -> Closed (2026-10-26T23:39:42Z, deltic:manual role=verify run=verify-20261026T225713Z-18640195 verified_fix_run=fix-20261023T021635Z-5347512d verdict=close)

## Observation

Confirmed compile; compiler.rs:792. A crafted fetch path becomes JS code; hyphenated module names produce invalid identifiers.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:39:42Z, deltic:manual run=verify-20261026T225713Z-18640195 verified_fix_run=fix-20261023T021635Z-5347512d verdict=close)

Verifier note: Manual verify: [lb luna] 3e24904; parity_js_import_path_not_injected and compiler::test_import_path_is_escaped_and_binding_mangled passed; unescaped-path mutant printed INJECTED; unsanitized-binding mutant failed expected `let my_mod = ...` assertion.

## Notes

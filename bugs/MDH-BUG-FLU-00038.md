# MDH-BUG-FLU-00038 — JS compiler splices import paths into require() unescaped

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T021635Z-5347512d
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00038-run-fix-20261023T021635Z-5347512d
- **Owner base:** 73e3d93e37bfee986402b27699ccfcf1da2b9b0d
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T02:16:35Z
- **Owner until:** 2026-10-23T06:16:35Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed compile; compiler.rs:792. A crafted fetch path becomes JS code; hyphenated module names produce invalid identifiers.

## Fix

<unfixed — raised only>

## Notes

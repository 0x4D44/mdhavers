# MDH-BUG-FLU-00038 — JS compiler splices import paths into require() unescaped

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T225713Z-18640195
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00038-run-verify-20261026T225713Z-18640195
- **Owner base:** 6cca1b9724e089b14af0018739faf304b5548a5a
- **Owner fingerprint:** sha256:e41f2b0e61b1455df5988cbf63cf037c0b1f37911a2e15157da13e359fc86a9e
- **Owner since:** 2026-10-26T22:57:13Z
- **Owner until:** 2026-10-27T02:57:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:16:41Z, deltic:auto role=fix run=fix-20261023T021635Z-5347512d branch=task/bug-MDH-BUG-FLU-00038-run-fix-20261023T021635Z-5347512d code=3e24904 gate=manual)

## Observation

Confirmed compile; compiler.rs:792. A crafted fetch path becomes JS code; hyphenated module names produce invalid identifiers.

## Fix

<unfixed — raised only>

## Notes

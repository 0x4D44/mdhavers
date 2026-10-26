# MDH-BUG-FLU-00035 — compile/build with an extension-less input can overwrite the source file

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** cli
- **Raised:** 2026-10-22T23:24:09Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T224201Z-b592dbb9
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00035-run-verify-20261026T224201Z-b592dbb9
- **Owner base:** 5122a4de097a3cb949560c57a78e74948858cacb
- **Owner fingerprint:** sha256:d2f95eb82476f990aea5eebee17bc8c4eaa08ea4b8b2990113f25e54cbbee52d
- **Owner since:** 2026-10-26T22:42:01Z
- **Owner until:** 2026-10-27T02:42:01Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:49:09Z, deltic:auto role=fix run=fix-20261023T024903Z-5b5dbd7f branch=task/bug-MDH-BUG-FLU-00035-run-fix-20261023T024903Z-5b5dbd7f code=77f950a gate=manual)

## Observation

Inferred: main.rs:521-525 and 370-376 derive the output path by stripping the extension; with none, output == input.

## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00035 — compile/build with an extension-less input can overwrite the source file

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** cli
- **Raised:** 2026-10-22T23:24:09Z
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
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:49:09Z, deltic:auto role=fix run=fix-20261023T024903Z-5b5dbd7f branch=task/bug-MDH-BUG-FLU-00035-run-fix-20261023T024903Z-5b5dbd7f code=77f950a gate=manual)

## Observation

Inferred: main.rs:521-525 and 370-376 derive the output path by stripping the extension; with none, output == input.

## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00034 — REPL `vars` panics on non-ASCII values

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
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:48:57Z, deltic:auto role=fix run=fix-20261023T024851Z-16839b1a branch=task/bug-MDH-BUG-FLU-00034-run-fix-20261023T024851Z-16839b1a code=77f950a gate=manual)

## Observation

Inferred (high confidence): main.rs:1151 slices &value[..47] by bytes; panics when byte 47 is inside a multi-byte char.

## Fix

<unfixed — raised only>

## Notes

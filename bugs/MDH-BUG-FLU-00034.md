# MDH-BUG-FLU-00034 — REPL `vars` panics on non-ASCII values

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** cli
- **Raised:** 2026-10-22T23:24:09Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T224201Z-50bd7d8f
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00034-run-verify-20261026T224201Z-50bd7d8f
- **Owner base:** 5122a4de097a3cb949560c57a78e74948858cacb
- **Owner fingerprint:** sha256:67f22314bcd529e6b75a07f8fb63802bb4fe1ac6bffd19e7a83531f5b306e5d6
- **Owner since:** 2026-10-26T22:42:01Z
- **Owner until:** 2026-10-27T02:42:01Z
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

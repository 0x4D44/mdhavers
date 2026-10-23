# MDH-BUG-FLU-00034 — REPL `vars` panics on non-ASCII values

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** cli
- **Raised:** 2026-10-22T23:24:09Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T024851Z-16839b1a
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00034-run-fix-20261023T024851Z-16839b1a
- **Owner base:** 3eb3ef021bbfbb31d62480877c90fe79a688e63a
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T02:48:51Z
- **Owner until:** 2026-10-23T06:48:51Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred (high confidence): main.rs:1151 slices &value[..47] by bytes; panics when byte 47 is inside a multi-byte char.

## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00034 — REPL `vars` panics on non-ASCII values

- **State:** Closed
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
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:48:57Z, deltic:auto role=fix run=fix-20261023T024851Z-16839b1a branch=task/bug-MDH-BUG-FLU-00034-run-fix-20261023T024851Z-16839b1a code=77f950a gate=manual) -> Closed (2026-10-26T23:48:13Z, deltic:manual role=verify run=verify-20261026T224201Z-50bd7d8f verified_fix_run=fix-20261023T024851Z-16839b1a verdict=close)

## Observation

Inferred (high confidence): main.rs:1151 slices &value[..47] by bytes; panics when byte 47 is inside a multi-byte char.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:48:13Z, deltic:manual run=verify-20261026T224201Z-50bd7d8f verified_fix_run=fix-20261023T024851Z-16839b1a verdict=close)

Verifier note: Manual verify: [la luna] 77f950a; REPL with 60 `é` characters and :vars exits 0 and displays truncation. cargo test --bin mdhavers tests selected 30 pass; byte-slice mutant failed truncate_for_display_cuts_on_char_boundary: byte 47 is inside é. Source restored clean.

## Notes

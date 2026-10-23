# MDH-BUG-FLU-00045 — WAT string literals with non-ASCII characters are corrupted

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** wasm
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T025747Z-1f8a1cd5
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00045-run-fix-20261023T025747Z-1f8a1cd5
- **Owner base:** b5d878d14de82ed6f562957cd7186df7f037df00
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T02:57:47Z
- **Owner until:** 2026-10-23T06:57:47Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred: escape_wat_string (wasm_compiler.rs:1152) writes \xx of the code point instead of UTF-8 bytes while passing the byte length.

## Fix

<unfixed — raised only>

## Notes

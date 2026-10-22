# MDH-BUG-FLU-00045 — WAT string literals with non-ASCII characters are corrupted

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** wasm
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
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred: escape_wat_string (wasm_compiler.rs:1152) writes \xx of the code point instead of UTF-8 bytes while passing the byte length.

## Fix

<unfixed — raised only>

## Notes

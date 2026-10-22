# MDH-BUG-FLU-00044 — compile_to_wat returns Ok for invalid modules

- **State:** Open
- **Priority:** Should
- **Severity:** High
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

Inferred (high confidence): functions-only programs export a missing $main (wasm_compiler.rs:560-566, the README example); user `dae main` duplicates $main; default params and unknown callees (any builtin) and top-level vars used in functions produce unresolved ids. No test validates the WAT.

## Fix

<unfixed — raised only>

## Notes

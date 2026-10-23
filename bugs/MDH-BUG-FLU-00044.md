# MDH-BUG-FLU-00044 — compile_to_wat returns Ok for invalid modules

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** wasm
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T025736Z-56aedf7c
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00044-run-fix-20261023T025736Z-56aedf7c
- **Owner base:** 3658149cadee0ffb29428e2026f51c9e89b21f62
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T02:57:36Z
- **Owner until:** 2026-10-23T06:57:36Z
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

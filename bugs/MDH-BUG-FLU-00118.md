# MDH-BUG-FLU-00118 — WASM: builtins that print with println! bypass the runner's captured stdout

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T03:37:13Z
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
- **State history:** Open (2026-10-24T03:37:13Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane W2 2026-10-24: natives called through __mdh_builtinN that write via println! go straight to the process stdout instead of the wasm_runner's capture, so ordering and capture differ from blether output under wasm-run and in tests.

Evidence fingerprint: `manual:v1:wasm-builtins-that-print-with-println-bypass-th-b782dcde0de54ea3`


## Fix

<unfixed — raised only>

## Notes

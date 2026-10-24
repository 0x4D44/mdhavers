# MDH-BUG-FLU-00118 — WASM: builtins that print with println! bypass the runner's captured stdout

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T03:37:13Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T035154Z-8f78a9bb
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00118-run-fix-20261024T035154Z-8f78a9bb
- **Owner base:** 2f95db36ecd2639d945e817201a663cade90c623
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T03:51:54Z
- **Owner until:** 2026-10-24T07:51:54Z
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

# MDH-BUG-FLU-00118 — WASM: builtins that print with println! bypass the runner's captured stdout

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T03:37:13Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261027T001425Z-794d1f1a
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00118-run-verify-20261027T001425Z-794d1f1a
- **Owner base:** bb18b71792fa6c1bc5f55d254da1f174c2d0af01
- **Owner fingerprint:** sha256:8dee3075e189116364c5ef422635f25a7f217b63185901b7ed59698133356b07
- **Owner since:** 2026-10-27T00:14:25Z
- **Owner until:** 2026-10-27T04:14:25Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T03:37:13Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T03:52:00Z, deltic:auto role=fix run=fix-20261024T035154Z-8f78a9bb branch=task/bug-MDH-BUG-FLU-00118-run-fix-20261024T035154Z-8f78a9bb code=7bf8799 gate=manual)

## Observation

Found by lane W2 2026-10-24: natives called through __mdh_builtinN that write via println! go straight to the process stdout instead of the wasm_runner's capture, so ordering and capture differ from blether output under wasm-run and in tests.

Evidence fingerprint: `manual:v1:wasm-builtins-that-print-with-println-bypass-th-b782dcde0de54ea3`


## Fix

<unfixed — raised only>

## Notes

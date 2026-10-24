# MDH-BUG-FLU-00121 — WASM host store has no bytes value

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T04:08:13Z
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
- **State history:** Open (2026-10-24T04:08:13Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T05:06:47Z, deltic:auto role=fix run=fix-20261024T050641Z-f6c686d0 branch=task/bug-MDH-BUG-FLU-00121-run-fix-20261024T050641Z-f6c686d0 code=cce0f83 gate=manual)

## Observation

Found by lane W3 2026-10-24: bytes builtins cannot round-trip through the wasm_runner host store, so bytes_oob_and_slice_bounds_parity is allowlisted in tests/wasm_validity_allowlist.txt.

Evidence fingerprint: `manual:v1:wasm-host-store-has-no-bytes-value`


## Fix

<unfixed — raised only>

## Notes

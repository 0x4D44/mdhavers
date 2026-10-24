# MDH-BUG-FLU-00121 — WASM host store has no bytes value

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T04:08:13Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T050641Z-f6c686d0
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00121-run-fix-20261024T050641Z-f6c686d0
- **Owner base:** 9d497ff20b9fb27f8dc9b6bc99550d1a6deca3b0
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T05:06:41Z
- **Owner until:** 2026-10-24T09:06:41Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T04:08:13Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane W3 2026-10-24: bytes builtins cannot round-trip through the wasm_runner host store, so bytes_oob_and_slice_bounds_parity is allowlisted in tests/wasm_validity_allowlist.txt.

Evidence fingerprint: `manual:v1:wasm-host-store-has-no-bytes-value`


## Fix

<unfixed — raised only>

## Notes

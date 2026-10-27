# MDH-BUG-FLU-00121 — WASM host store has no bytes value

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T04:08:13Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261027T001425Z-d9bf2fea
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00121-run-verify-20261027T001425Z-d9bf2fea
- **Owner base:** bb18b71792fa6c1bc5f55d254da1f174c2d0af01
- **Owner fingerprint:** sha256:62337dc74c4c401b2765c1c11c2176f5fb2b6ede82f0fdbb0ec05cbfb20cb261
- **Owner since:** 2026-10-27T00:14:25Z
- **Owner until:** 2026-10-27T04:14:25Z
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

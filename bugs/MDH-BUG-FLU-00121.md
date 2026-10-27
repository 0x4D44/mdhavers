# MDH-BUG-FLU-00121 — WASM host store has no bytes value

- **State:** Closed
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
- **State history:** Open (2026-10-24T04:08:13Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T05:06:47Z, deltic:auto role=fix run=fix-20261024T050641Z-f6c686d0 branch=task/bug-MDH-BUG-FLU-00121-run-fix-20261024T050641Z-f6c686d0 code=cce0f83 gate=manual) -> Closed (2026-10-27T01:32:14Z, deltic:manual role=verify run=verify-20261027T001425Z-d9bf2fea verified_fix_run=fix-20261024T050641Z-f6c686d0 verdict=close)

## Observation

Found by lane W3 2026-10-24: bytes builtins cannot round-trip through the wasm_runner host store, so bytes_oob_and_slice_bounds_parity is allowlisted in tests/wasm_validity_allowlist.txt.

Evidence fingerprint: `manual:v1:wasm-host-store-has-no-bytes-value`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:32:14Z, deltic:manual run=verify-20261027T001425Z-d9bf2fea verified_fix_run=fix-20261024T050641Z-f6c686d0 verdict=close)

Verifier note: Manual verify: [ld luna] fix e0c5eb1; bytes_round_trip_through_the_host_store passed; rejecting HostValue::Bytes failed its no-runtime-error assertion with "WASM cannae haud bytes yet".

## Notes

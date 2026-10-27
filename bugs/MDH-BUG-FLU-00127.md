# MDH-BUG-FLU-00127 — WASM builtin bridge cannot carry function values (dict of lambdas to contains)

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T06:14:54Z
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
- **State history:** Open (2026-10-24T06:14:54Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T06:41:54Z, deltic:auto role=fix run=fix-20261024T064147Z-a4a054ee branch=task/bug-MDH-BUG-FLU-00127-run-fix-20261024T064147Z-a4a054ee code=dae3c18 gate=manual) -> Closed (2026-10-27T01:32:14Z, deltic:manual role=verify run=verify-20261027T001425Z-490a14ea verified_fix_run=fix-20261024T064147Z-a4a054ee verdict=close)

## Observation

Found by lane W7 (c436abd) 2026-10-24: to_value in src/wasm_runner.rs cannot convert a host function value to an interpreter Value, so passing a dict or list holding lambdas to a builtin (contains in tests/golden/integration/showcase_bothy_adventure.braw) fails. The last entry in tests/wasm_parity_allowlist.txt. Needs a function-value Value variant that round-trips through the bridge and write-back.

Evidence fingerprint: `manual:v1:wasm-builtin-bridge-cannot-carry-function-value-bc75ae7c474a9817`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:32:14Z, deltic:manual run=verify-20261027T001425Z-490a14ea verified_fix_run=fix-20261024T064147Z-a4a054ee verdict=close)

Verifier note: Manual verify: [ld luna] fix dae3c18 (source 8254242); wasm_golden_parity passed 263 fixtures; rejecting HostValue::Function made showcase_bothy_adventure.braw fail parity with output mismatch.

## Notes

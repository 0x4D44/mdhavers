# MDH-BUG-FLU-00127 — WASM builtin bridge cannot carry function values (dict of lambdas to contains)

- **State:** Open
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
- **State history:** Open (2026-10-24T06:14:54Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane W7 (c436abd) 2026-10-24: to_value in src/wasm_runner.rs cannot convert a host function value to an interpreter Value, so passing a dict or list holding lambdas to a builtin (contains in tests/golden/integration/showcase_bothy_adventure.braw) fails. The last entry in tests/wasm_parity_allowlist.txt. Needs a function-value Value variant that round-trips through the bridge and write-back.

Evidence fingerprint: `manual:v1:wasm-builtin-bridge-cannot-carry-function-value-bc75ae7c474a9817`


## Fix

<unfixed — raised only>

## Notes

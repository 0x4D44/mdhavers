# MDH-BUG-FLU-00122 — WASM prints non-string dict keys in Rust debug form

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T04:51:03Z
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
- **State history:** Open (2026-10-24T04:51:03Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T05:07:00Z, deltic:auto role=fix run=fix-20261024T050653Z-0ae1cee1 branch=task/bug-MDH-BUG-FLU-00122-run-fix-20261024T050653Z-0ae1cee1 code=cce0f83 gate=manual)

## Observation

Found by lane W4 2026-10-24, confirmed by the lead (executed): blether grup_up([1, 2], |x| x % 2) prints {"Int(1)": [1], "Int(0)": [2]} under wasm-run but {"1": [1], "0": [2]} in the interpreter. Cause: HostStore::to_string in src/wasm_runner.rs formats non-string dict keys with format!("{:?}", key).

Evidence fingerprint: `manual:v1:wasm-prints-non-string-dict-keys-in-rust-debug-form`


## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00124 — WASM host may display creel string members without quotes (unverified)

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T05:07:06Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T052930Z-27a080f9
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00124-run-fix-20261024T052930Z-27a080f9
- **Owner base:** 06978e5c44b260d0cd35930ac6a0d9054a75d2ab
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T05:29:30Z
- **Owner until:** 2026-10-24T09:29:30Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T05:07:06Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Reported by lane W5 2026-10-24 by reading HostStore::to_string (src/wasm_runner.rs), not executed: creel members may print unquoted under wasm-run while the interpreter prints creel{"1"}. Verify with: ken c = creel(["1", 2])  blether c  on run vs wasm then wasm-run; close as not-a-bug if they match.

Evidence fingerprint: `manual:v1:wasm-host-may-display-creel-string-members-with-d9a9bb2cc9db434d`


## Fix

<unfixed — raised only>

## Notes

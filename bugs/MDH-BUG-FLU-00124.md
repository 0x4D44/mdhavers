# MDH-BUG-FLU-00124 — WASM host may display creel string members without quotes (unverified)

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T05:07:06Z
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
- **State history:** Open (2026-10-24T05:07:06Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Reported by lane W5 2026-10-24 by reading HostStore::to_string (src/wasm_runner.rs), not executed: creel members may print unquoted under wasm-run while the interpreter prints creel{"1"}. Verify with: ken c = creel(["1", 2])  blether c  on run vs wasm then wasm-run; close as not-a-bug if they match.

Evidence fingerprint: `manual:v1:wasm-host-may-display-creel-string-members-with-d9a9bb2cc9db434d`


## Fix

<unfixed — raised only>

## Notes

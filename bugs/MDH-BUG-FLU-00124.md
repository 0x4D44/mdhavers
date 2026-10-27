# MDH-BUG-FLU-00124 — WASM host may display creel string members without quotes (unverified)

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T05:07:06Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261027T001425Z-0a5be9b5
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00124-run-verify-20261027T001425Z-0a5be9b5
- **Owner base:** bb18b71792fa6c1bc5f55d254da1f174c2d0af01
- **Owner fingerprint:** sha256:682ccf31419051192ed3b3b25a363f8189c8e6370ab2632ac1ae807368fff363
- **Owner since:** 2026-10-27T00:14:25Z
- **Owner until:** 2026-10-27T04:14:25Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T05:07:06Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T05:29:36Z, deltic:auto role=fix run=fix-20261024T052930Z-27a080f9 branch=task/bug-MDH-BUG-FLU-00124-run-fix-20261024T052930Z-27a080f9 code=06978e5 gate=manual)

## Observation

Reported by lane W5 2026-10-24 by reading HostStore::to_string (src/wasm_runner.rs), not executed: creel members may print unquoted under wasm-run while the interpreter prints creel{"1"}. Verify with: ken c = creel(["1", 2])  blether c  on run vs wasm then wasm-run; close as not-a-bug if they match.

Evidence fingerprint: `manual:v1:wasm-host-may-display-creel-string-members-with-d9a9bb2cc9db434d`


## Fix

<unfixed — raised only>

## Notes

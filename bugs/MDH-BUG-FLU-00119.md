# MDH-BUG-FLU-00119 — WASM: a builtin returning a creel stops the program (host store cannot hold a set)

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T03:37:18Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261027T001425Z-898c3845
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00119-run-verify-20261027T001425Z-898c3845
- **Owner base:** bb18b71792fa6c1bc5f55d254da1f174c2d0af01
- **Owner fingerprint:** sha256:a9d2263be0ec5039613dadb07620d24b14e043db09e37d2f131a323464f09c61
- **Owner since:** 2026-10-27T00:14:25Z
- **Owner until:** 2026-10-27T04:14:25Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T03:37:18Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T03:52:10Z, deltic:auto role=fix run=fix-20261024T035205Z-64217157 branch=task/bug-MDH-BUG-FLU-00119-run-fix-20261024T035205Z-64217157 code=7bf8799 gate=manual)

## Observation

Found by lane W2 2026-10-24: the wasm_runner host store has no set representation, so creel-returning natives raise "WASM cannae haud a creel yet"; 3 goldens are allowlisted for it.

Evidence fingerprint: `manual:v1:wasm-a-builtin-returning-a-creel-stops-the-prog-caa67f30f18c6be2`


## Fix

<unfixed — raised only>

## Notes

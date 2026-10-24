# MDH-BUG-FLU-00119 — WASM: a builtin returning a creel stops the program (host store cannot hold a set)

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T03:37:18Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T035205Z-64217157
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00119-run-fix-20261024T035205Z-64217157
- **Owner base:** f84b1ff24de710811f7d7bba3fca4fcdef0ba3b4
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T03:52:05Z
- **Owner until:** 2026-10-24T07:52:05Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T03:37:18Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane W2 2026-10-24: the wasm_runner host store has no set representation, so creel-returning natives raise "WASM cannae haud a creel yet"; 3 goldens are allowlisted for it.

Evidence fingerprint: `manual:v1:wasm-a-builtin-returning-a-creel-stops-the-prog-caa67f30f18c6be2`


## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00117 — WASM builtin calls convert whole arguments each call, so len(xs) in a loop is quadratic

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T03:37:09Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261027T001425Z-942496d7
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00117-run-verify-20261027T001425Z-942496d7
- **Owner base:** bb18b71792fa6c1bc5f55d254da1f174c2d0af01
- **Owner fingerprint:** sha256:db974221b541ae9ded9e9241d3ec6642a83d242e9af201e4d4062595e1e052cf
- **Owner since:** 2026-10-27T00:14:25Z
- **Owner until:** 2026-10-27T04:14:25Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T03:37:09Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T03:51:49Z, deltic:auto role=fix run=fix-20261024T035144Z-f72226af branch=task/bug-MDH-BUG-FLU-00117-run-fix-20261024T035144Z-f72226af code=7bf8799 gate=manual)

## Observation

Found by lane W2 (4194046) 2026-10-24: call_builtin in src/wasm_runner.rs converts every argument handle to a Value and checks containers for write-back after each call, costing O(size of args). A loop calling len(xs) over xs is O(n^2). Add a fast path for scalar-only arguments and for read-only natives (len, keys, heid, ...). Deeply nested arguments also convert recursively on the native stack.

Evidence fingerprint: `manual:v1:wasm-builtin-calls-convert-whole-arguments-each-a531878a0fe4aab2`


## Fix

<unfixed — raised only>

## Notes

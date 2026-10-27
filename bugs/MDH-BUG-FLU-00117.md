# MDH-BUG-FLU-00117 — WASM builtin calls convert whole arguments each call, so len(xs) in a loop is quadratic

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T03:37:09Z
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
- **State history:** Open (2026-10-24T03:37:09Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T03:51:49Z, deltic:auto role=fix run=fix-20261024T035144Z-f72226af branch=task/bug-MDH-BUG-FLU-00117-run-fix-20261024T035144Z-f72226af code=7bf8799 gate=manual) -> Closed (2026-10-27T01:32:14Z, deltic:manual role=verify run=verify-20261027T001425Z-942496d7 verified_fix_run=fix-20261024T035144Z-f72226af verdict=close)

## Observation

Found by lane W2 (4194046) 2026-10-24: call_builtin in src/wasm_runner.rs converts every argument handle to a Value and checks containers for write-back after each call, costing O(size of args). A loop calling len(xs) over xs is O(n^2). Add a fast path for scalar-only arguments and for read-only natives (len, keys, heid, ...). Deeply nested arguments also convert recursively on the native stack.

Evidence fingerprint: `manual:v1:wasm-builtin-calls-convert-whole-arguments-each-a531878a0fe4aab2`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:32:14Z, deltic:manual run=verify-20261027T001425Z-942496d7 verified_fix_run=fix-20261024T035144Z-f72226af verdict=close)

Verifier note: Manual verify: [ld luna] fix 819568f; len_in_a_loop_over_a_big_list_is_not_quadratic passed (16.69s); per-handle store.view traversal mutant failed the <20s assertion at 102.93s.

## Notes

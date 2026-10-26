# MDH-BUG-FLU-00112 — Native: a dae nested inside a gin block cannot capture its enclosing function's parameter

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-24T02:39:09Z
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
- **State history:** Open (2026-10-24T02:39:09Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T03:04:18Z, deltic:auto role=fix run=fix-20261024T030413Z-e8a85c1e branch=task/bug-MDH-BUG-FLU-00112-run-fix-20261024T030413Z-e8a85c1e code=efb4bb2 gate=manual) -> Closed (2026-10-26T19:40:44Z, deltic:manual role=verify run=verify-20261026T182021Z-a3118833 verified_fix_run=fix-20261024T030413Z-e8a85c1e verdict=close)

## Observation

Found by lane N3 2026-10-24 (executed, also on trunk before e92c292). dae outer(v) { gin v > 0 { dae inner() { gie v * 10 }  gie inner() }  gie v }  blether outer(3) -> interpreter 30, native compile error "Undefined variable: v". Likely compile_stmt Stmt::Function (~src/llvm/codegen.rs 13043) declares non-top-of-body nested functions without captures.

Evidence fingerprint: `manual:v1:native-a-dae-nested-inside-a-gin-block-cannot-c-9691cab58d79c6ec`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T19:40:44Z, deltic:manual run=verify-20261026T182021Z-a3118833 verified_fix_run=fix-20261024T030413Z-e8a85c1e verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] fix=528b015 (history code efb4bb2 is version bump); interpreter/native repro both printed 30; dae_in_a_gin_block_captures_the_enclosing_parameter passed 1/1; mutant replaced captured cell with nil and test assertion on run.code==Some(0) failed (None).

## Notes

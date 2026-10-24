# MDH-BUG-FLU-00112 — Native: a dae nested inside a gin block cannot capture its enclosing function's parameter

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-24T02:39:09Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T030413Z-e8a85c1e
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00112-run-fix-20261024T030413Z-e8a85c1e
- **Owner base:** 13d555b2fb3a3825c4ed002d875ea43947350e38
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T03:04:13Z
- **Owner until:** 2026-10-24T07:04:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T02:39:09Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane N3 2026-10-24 (executed, also on trunk before e92c292). dae outer(v) { gin v > 0 { dae inner() { gie v * 10 }  gie inner() }  gie v }  blether outer(3) -> interpreter 30, native compile error "Undefined variable: v". Likely compile_stmt Stmt::Function (~src/llvm/codegen.rs 13043) declares non-top-of-body nested functions without captures.

Evidence fingerprint: `manual:v1:native-a-dae-nested-inside-a-gin-block-cannot-c-9691cab58d79c6ec`


## Fix

<unfixed — raised only>

## Notes

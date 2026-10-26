# MDH-BUG-FLU-00112 — Native: a dae nested inside a gin block cannot capture its enclosing function's parameter

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-24T02:39:09Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T182021Z-a3118833
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00112-run-verify-20261026T182021Z-a3118833
- **Owner base:** 2f352f49ef4c71b45a0497bf81a04779c622faad
- **Owner fingerprint:** sha256:b48adf764943257b3d540a77c054662cf9a09d45776b9eb14ebeb71d2d9bac48
- **Owner since:** 2026-10-26T18:20:21Z
- **Owner until:** 2026-10-26T22:20:21Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T02:39:09Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T03:04:18Z, deltic:auto role=fix run=fix-20261024T030413Z-e8a85c1e branch=task/bug-MDH-BUG-FLU-00112-run-fix-20261024T030413Z-e8a85c1e code=efb4bb2 gate=manual)

## Observation

Found by lane N3 2026-10-24 (executed, also on trunk before e92c292). dae outer(v) { gin v > 0 { dae inner() { gie v * 10 }  gie inner() }  gie v }  blether outer(3) -> interpreter 30, native compile error "Undefined variable: v". Likely compile_stmt Stmt::Function (~src/llvm/codegen.rs 13043) declares non-top-of-body nested functions without captures.

Evidence fingerprint: `manual:v1:native-a-dae-nested-inside-a-gin-block-cannot-c-9691cab58d79c6ec`


## Fix

<unfixed — raised only>

## Notes

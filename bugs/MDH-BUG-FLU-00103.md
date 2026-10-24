# MDH-BUG-FLU-00103 — Native: segfault when a lambda in a gin branch captures a parameter used after the branch

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** native
- **Raised:** 2026-10-24T02:09:51Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T023837Z-7167b83e
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00103-run-fix-20261024T023837Z-7167b83e
- **Owner base:** 21a9fafd8ecdcbf64b433c5ab5b0fd5c32dfc014
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T02:38:37Z
- **Owner until:** 2026-10-24T06:38:37Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T02:09:51Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane X1 2026-10-24. Repro: dae f(v) { gin v == 0 { gie gaun(["a"], |key| v) } gie v }  blether f(5) : interpreter and JS print 5, native segfaults (f(0) works). Inference from code: ensure_boxed_variable (src/llvm/codegen.rs ~9061) boxes the variable only on the branch that compiles the lambda but marks it boxed function-wide, so the other path reads a plain int as a box. Golden tests/golden/integration/showcase_config_differ.braw is SKIP_NATIVE on this.

Evidence fingerprint: `manual:v1:native-segfault-when-a-lambda-in-a-gin-branch-c-bf2968ffd09e67de`


## Fix

<unfixed — raised only>

## Notes

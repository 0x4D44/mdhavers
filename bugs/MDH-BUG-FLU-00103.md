# MDH-BUG-FLU-00103 — Native: segfault when a lambda in a gin branch captures a parameter used after the branch

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** native
- **Raised:** 2026-10-24T02:09:51Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T164142Z-b8ac7b72
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00103-run-verify-20261026T164142Z-b8ac7b72
- **Owner base:** b5d5a9fa852316d9d5ef58b28e50e2449fbe0bad
- **Owner fingerprint:** sha256:40bf2187af2efaa4c96464bc97543bdcf366c0fc5208b320eb10e3999d08d88a
- **Owner since:** 2026-10-26T16:41:42Z
- **Owner until:** 2026-10-26T20:41:42Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T02:09:51Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T02:38:43Z, deltic:auto role=fix run=fix-20261024T023837Z-7167b83e branch=task/bug-MDH-BUG-FLU-00103-run-fix-20261024T023837Z-7167b83e code=e92c292 gate=manual)

## Observation

Found by lane X1 2026-10-24. Repro: dae f(v) { gin v == 0 { gie gaun(["a"], |key| v) } gie v }  blether f(5) : interpreter and JS print 5, native segfaults (f(0) works). Inference from code: ensure_boxed_variable (src/llvm/codegen.rs ~9061) boxes the variable only on the branch that compiles the lambda but marks it boxed function-wide, so the other path reads a plain int as a box. Golden tests/golden/integration/showcase_config_differ.braw is SKIP_NATIVE on this.

Evidence fingerprint: `manual:v1:native-segfault-when-a-lambda-in-a-gin-branch-c-bf2968ffd09e67de`


## Fix

<unfixed — raised only>

## Notes

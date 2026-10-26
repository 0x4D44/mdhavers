# MDH-BUG-FLU-00135 — Native: a dae nested in a lambda body cannot read the lambda's locals

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T02:34:36Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T182021Z-d550e7fe
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00135-run-verify-20261026T182021Z-d550e7fe
- **Owner base:** 2f352f49ef4c71b45a0497bf81a04779c622faad
- **Owner fingerprint:** sha256:c72d8fecd16a3a345590c3deb5f7e7d020a464acc23c10ece59821c5be0f9f7c
- **Owner since:** 2026-10-26T18:20:21Z
- **Owner until:** 2026-10-26T22:20:21Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T02:34:36Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T03:00:50Z, deltic:auto role=fix run=fix-20261025T023534Z-b8acd3fc branch=task/bug-MDH-BUG-FLU-00135-run-fix-20261025T023534Z-b8acd3fc code=d6b6308 gate=manual)

## Observation

Found by lane S7 2026-10-25 (executed) via the coverage-only test llvm_codegen_lambda_freevar_analysis_exercises_more_stmt_and_expr_kinds (tests/llvm_codegen_lambda_freevars_coverage.rs), which now fails with CompileError("Undefined variable: out"). Repro: ken f = |p| { ken out = p + 1  dae inner() { gie out }  gie out }  blether f(5) -> mdhavers run prints 6; native build fails "Undefined variable: out". The same nested dae inside a named dae works natively. Suspected (unconfirmed): the lambda free-variable scan does not recurse into nested function bodies, so out is never captured for inner.

Evidence fingerprint: `manual:v1:native-a-dae-nested-in-a-lambda-body-cannot-rea-f0db5840290d7e95`


## Fix

<unfixed — raised only>

## Notes

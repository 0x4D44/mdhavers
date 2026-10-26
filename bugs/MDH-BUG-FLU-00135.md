# MDH-BUG-FLU-00135 — Native: a dae nested in a lambda body cannot read the lambda's locals

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T02:34:36Z
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
- **State history:** Open (2026-10-25T02:34:36Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T03:00:50Z, deltic:auto role=fix run=fix-20261025T023534Z-b8acd3fc branch=task/bug-MDH-BUG-FLU-00135-run-fix-20261025T023534Z-b8acd3fc code=d6b6308 gate=manual) -> Closed (2026-10-26T19:40:44Z, deltic:manual role=verify run=verify-20261026T182021Z-d550e7fe verified_fix_run=fix-20261025T023534Z-b8acd3fc verdict=close)

## Observation

Found by lane S7 2026-10-25 (executed) via the coverage-only test llvm_codegen_lambda_freevar_analysis_exercises_more_stmt_and_expr_kinds (tests/llvm_codegen_lambda_freevars_coverage.rs), which now fails with CompileError("Undefined variable: out"). Repro: ken f = |p| { ken out = p + 1  dae inner() { gie out }  gie out }  blether f(5) -> mdhavers run prints 6; native build fails "Undefined variable: out". The same nested dae inside a named dae works natively. Suspected (unconfirmed): the lambda free-variable scan does not recurse into nested function bodies, so out is never captured for inner.

Evidence fingerprint: `manual:v1:native-a-dae-nested-in-a-lambda-body-cannot-rea-f0db5840290d7e95`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T19:40:44Z, deltic:manual run=verify-20261026T182021Z-d550e7fe verified_fix_run=fix-20261025T023534Z-b8acd3fc verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] fix=d6b6308; interpreter/native repro both printed 6; dae_nested_in_a_lambda_reads_the_lambdas_scope passed 1/1; mutant passed nil instead of captured cells and test assertion run.code==Some(0) failed (None).

## Notes

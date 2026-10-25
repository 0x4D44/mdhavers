# MDH-BUG-FLU-00135 — Native: a dae nested in a lambda body cannot read the lambda's locals

- **State:** Open
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
- **State history:** Open (2026-10-25T02:34:36Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane S7 2026-10-25 (executed) via the coverage-only test llvm_codegen_lambda_freevar_analysis_exercises_more_stmt_and_expr_kinds (tests/llvm_codegen_lambda_freevars_coverage.rs), which now fails with CompileError("Undefined variable: out"). Repro: ken f = |p| { ken out = p + 1  dae inner() { gie out }  gie out }  blether f(5) -> mdhavers run prints 6; native build fails "Undefined variable: out". The same nested dae inside a named dae works natively. Suspected (unconfirmed): the lambda free-variable scan does not recurse into nested function bodies, so out is never captured for inner.

Evidence fingerprint: `manual:v1:native-a-dae-nested-in-a-lambda-body-cannot-rea-f0db5840290d7e95`


## Fix

<unfixed — raised only>

## Notes

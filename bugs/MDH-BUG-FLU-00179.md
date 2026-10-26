# MDH-BUG-FLU-00179 — Native: a nested function reads the global, not an enclosing function-local of the same name

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-26T06:20:04Z
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
- **State history:** Open (2026-10-26T06:20:04Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T09:13:57Z, deltic:auto role=fix run=fix-20261026T065014Z-226467ae branch=task/bug-MDH-BUG-FLU-00179-run-fix-20261026T065014Z-226467ae code=a2a1e4d gate=manual)

## Observation

Found by lane N10 2026-10-26 (code read and probe): a nested dae that reads a function-level local which shadows a global name reads the global instead, because predeclare_local_name in src/llvm/codegen.rs skips names that are globals, so capture discovery never sees the local. E.g. ken x = 1; dae f() { ken x = 2; dae g() { gie x }; gie g() } returns 1 natively, 2 in the interpreter. Fix capture discovery to respect local shadowing; parity test incl. closures/lambdas.

Evidence fingerprint: `manual:v1:native-a-nested-function-reads-the-global-not-a-25009ef2698f1887`


## Fix

<unfixed — raised only>

## Notes

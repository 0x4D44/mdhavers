# MDH-BUG-FLU-00146 — Native prints function values as <object> instead of <dae name>

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-25T08:13:38Z
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
- **State history:** Open (2026-10-25T08:13:38Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T02:41:36Z, deltic:auto role=fix run=fix-20261026T014736Z-b0f67865 branch=task/bug-MDH-BUG-FLU-00146-run-fix-20261026T014736Z-b0f67865 code=0fb36d6 gate=manual) -> Closed (2026-10-26T19:40:44Z, deltic:manual role=verify run=verify-20261026T182021Z-b44dea60 verified_fix_run=fix-20261026T014736Z-b0f67865 verdict=close)

## Observation

Found by lane S10's builtin audit 2026-10-25 (executed): the interpreter displays a function value as <dae name> or <dae <lambda>>, native as <object>, because native function values carry no name at run time (runtime/mdh_runtime.c __mdh_value_to_string_sb default case). Affects tae_string, json_*, inspect, spy, swatch, clype and the Scots exclamation builtins whenever a function is printed. Needs a runtime name table or a name field on closures.

Evidence fingerprint: `manual:v1:native-prints-function-values-as-object-instead-of-dae-name`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T19:40:44Z, deltic:manual run=verify-20261026T182021Z-b44dea60 verified_fix_run=fix-20261026T014736Z-b0f67865 verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] fix=0fb36d6; named function/lambda repro matches interpreter/native; both llvm_function_names tests passed, including MDH_RC; mutant omitted emit_fn_name_table registration and both failed output assertions with named values rendered as <dae <lambda>>.

## Notes

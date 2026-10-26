# MDH-BUG-FLU-00103 — Native: segfault when a lambda in a gin branch captures a parameter used after the branch

- **State:** Closed
- **Priority:** Should
- **Severity:** High
- **Area:** native
- **Raised:** 2026-10-24T02:09:51Z
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
- **State history:** Open (2026-10-24T02:09:51Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T02:38:43Z, deltic:auto role=fix run=fix-20261024T023837Z-7167b83e branch=task/bug-MDH-BUG-FLU-00103-run-fix-20261024T023837Z-7167b83e code=e92c292 gate=manual) -> Closed (2026-10-26T17:16:10Z, deltic:manual role=verify run=verify-20261026T164142Z-b8ac7b72 verified_fix_run=fix-20261024T023837Z-7167b83e verdict=close)

## Observation

Found by lane X1 2026-10-24. Repro: dae f(v) { gin v == 0 { gie gaun(["a"], |key| v) } gie v }  blether f(5) : interpreter and JS print 5, native segfaults (f(0) works). Inference from code: ensure_boxed_variable (src/llvm/codegen.rs ~9061) boxes the variable only on the branch that compiles the lambda but marks it boxed function-wide, so the other path reads a plain int as a box. Golden tests/golden/integration/showcase_config_differ.braw is SKIP_NATIVE on this.

Evidence fingerprint: `manual:v1:native-segfault-when-a-lambda-in-a-gin-branch-c-bf2968ffd09e67de`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T17:16:10Z, deltic:manual run=verify-20261026T164142Z-b8ac7b72 verified_fix_run=fix-20261024T023837Z-7167b83e verdict=close)

Verifier note: Manual verify: [verifier gpt-6.1-sol (Codex), recorded by lead] e92c292 contains fix aaa6e6b: capture_on_one_branch_is_sound_on_every_path passed; original CLI/interpreter f(5) prints 5. Disabling eager captured-parameter boxing failed run_ok assertion None vs Some(0), recreating crash. Restored; diff empty.

## Notes

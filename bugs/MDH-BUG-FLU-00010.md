# MDH-BUG-FLU-00010 — Constructors and method calls skip arity checks

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:07Z
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
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:49:42Z, deltic:auto role=fix run=fix-20261023T014937Z-97820486 branch=task/bug-MDH-BUG-FLU-00010-run-fix-20261023T014937Z-97820486 code=d84823a gate=manual) -> Closed (2026-10-26T23:48:13Z, deltic:manual role=verify run=verify-20261026T224201Z-8c1f301a verified_fix_run=fix-20261023T014937Z-97820486 verdict=close)

## Observation

Confirmed: calling a kin constructor or obj.m() with too few args silently binds naething; extra args are dropped (interpreter.rs ~12252, ~11355; call_method_on_instance uses strict params.len() and ignores defaults ~12155).

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:48:13Z, deltic:manual run=verify-20261026T224201Z-8c1f301a verified_fix_run=fix-20261023T014937Z-97820486 verdict=close)

Verifier note: Manual verify: [la luna] d84823a; constructor_with_too_few_args, constructor_with_too_many_args, and method_with_too_many_args passed. Removing the shared arity check made all three fail with `expected an error, got output: []`. Restored; source diff clean.

## Notes

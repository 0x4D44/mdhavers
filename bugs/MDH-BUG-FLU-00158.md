# MDH-BUG-FLU-00158 — Native cannot use a builtin as a value: ken f = len fails to compile

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-26T02:17:22Z
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
- **State history:** Open (2026-10-26T02:17:22Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T03:56:41Z, deltic:auto role=fix run=fix-20261026T024153Z-9ac12aa3 branch=task/bug-MDH-BUG-FLU-00158-run-fix-20261026T024153Z-9ac12aa3 code=06cfc48 gate=manual) -> Closed (2026-10-26T21:44:08Z, deltic:manual role=verify run=verify-20261026T194306Z-4839c974 verified_fix_run=fix-20261026T024153Z-9ac12aa3 verdict=close)

## Observation

Found by lane S17 2026-10-26 (observed): natively, ken f = len; blether f fails at compile time with 'Undefined variable: len', where the interpreter prints <native dae len> and lets the value be called (e.g. passed to ilka/gaun). Programs that pass builtins to higher-order functions (gaun(xs, tae_string)) therefore don't build natively. Fix: wrap a builtin referenced as a value in a generated function value (per arity), named so display gives <native dae name>; parity test.

Evidence fingerprint: `manual:v1:native-cannot-use-a-builtin-as-a-value-ken-f-le-e1ff43199699af7d`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T21:44:08Z, deltic:manual run=verify-20261026T194306Z-4839c974 verified_fix_run=fix-20261026T024153Z-9ac12aa3 verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] 06cfc48; builtins_work_as_values_like_the_interpreter passes on trunk; a nil-return wrapper mutant failed its output parity assert (native values were naething; expected 3, 5, etc.).

## Notes

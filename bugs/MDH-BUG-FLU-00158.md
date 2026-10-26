# MDH-BUG-FLU-00158 — Native cannot use a builtin as a value: ken f = len fails to compile

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-26T02:17:22Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T194306Z-4839c974
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00158-run-verify-20261026T194306Z-4839c974
- **Owner base:** 36750bc8afd262a99873992d919d9ed250946aef
- **Owner fingerprint:** sha256:96f8c3e18f829850f8ee1b8d03486a7082181b0c3b05830b72f065caff927ced
- **Owner since:** 2026-10-26T19:43:06Z
- **Owner until:** 2026-10-26T23:43:06Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T02:17:22Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T03:56:41Z, deltic:auto role=fix run=fix-20261026T024153Z-9ac12aa3 branch=task/bug-MDH-BUG-FLU-00158-run-fix-20261026T024153Z-9ac12aa3 code=06cfc48 gate=manual)

## Observation

Found by lane S17 2026-10-26 (observed): natively, ken f = len; blether f fails at compile time with 'Undefined variable: len', where the interpreter prints <native dae len> and lets the value be called (e.g. passed to ilka/gaun). Programs that pass builtins to higher-order functions (gaun(xs, tae_string)) therefore don't build natively. Fix: wrap a builtin referenced as a value in a generated function value (per arity), named so display gives <native dae name>; parity test.

Evidence fingerprint: `manual:v1:native-cannot-use-a-builtin-as-a-value-ken-f-le-e1ff43199699af7d`


## Fix

<unfixed — raised only>

## Notes

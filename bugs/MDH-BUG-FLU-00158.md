# MDH-BUG-FLU-00158 — Native cannot use a builtin as a value: ken f = len fails to compile

- **State:** Open
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
- **State history:** Open (2026-10-26T02:17:22Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane S17 2026-10-26 (observed): natively, ken f = len; blether f fails at compile time with 'Undefined variable: len', where the interpreter prints <native dae len> and lets the value be called (e.g. passed to ilka/gaun). Programs that pass builtins to higher-order functions (gaun(xs, tae_string)) therefore don't build natively. Fix: wrap a builtin referenced as a value in a generated function value (per arity), named so display gives <native dae name>; parity test.

Evidence fingerprint: `manual:v1:native-cannot-use-a-builtin-as-a-value-ken-f-le-e1ff43199699af7d`


## Fix

<unfixed — raised only>

## Notes

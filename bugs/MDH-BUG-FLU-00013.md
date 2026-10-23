# MDH-BUG-FLU-00013 — Higher-order builtins are callable magic strings

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T040348Z-1113e70e
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00013-run-fix-20261023T040348Z-1113e70e
- **Owner base:** 7eeef3bf5c1cadfbf0aa7f53f6fb7a9d09c933df
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T04:03:48Z
- **Owner until:** 2026-10-23T08:03:48Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: whit_kind(gaun) is "string", blether gaun prints __builtin_gaun__, and any string with that prefix is callable (interpreter.rs ~7849, ~12238).

## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00146 — Native prints function values as <object> instead of <dae name>

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-25T08:13:38Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261026T014736Z-b0f67865
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00146-run-fix-20261026T014736Z-b0f67865
- **Owner base:** 13b942ee2e0dee4642622c1287daab60f96ab9d1
- **Owner fingerprint:** -
- **Owner since:** 2026-10-26T01:47:36Z
- **Owner until:** 2026-10-26T05:47:36Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T08:13:38Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane S10's builtin audit 2026-10-25 (executed): the interpreter displays a function value as <dae name> or <dae <lambda>>, native as <object>, because native function values carry no name at run time (runtime/mdh_runtime.c __mdh_value_to_string_sb default case). Affects tae_string, json_*, inspect, spy, swatch, clype and the Scots exclamation builtins whenever a function is printed. Needs a runtime name table or a name field on closures.

Evidence fingerprint: `manual:v1:native-prints-function-values-as-object-instead-of-dae-name`


## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00013 — Higher-order builtins are callable magic strings

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
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
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T04:03:54Z, deltic:auto role=fix run=fix-20261023T040348Z-1113e70e branch=task/bug-MDH-BUG-FLU-00013-run-fix-20261023T040348Z-1113e70e code=af1000d gate=manual) -> Closed (2026-10-26T23:48:13Z, deltic:manual role=verify run=verify-20261026T224201Z-df00531b verified_fix_run=fix-20261023T040348Z-1113e70e verdict=close)

## Observation

Confirmed: whit_kind(gaun) is "string", blether gaun prints __builtin_gaun__, and any string with that prefix is callable (interpreter.rs ~7849, ~12238).

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:48:13Z, deltic:manual run=verify-20261026T224201Z-df00531b verified_fix_run=fix-20261023T040348Z-1113e70e verdict=close)

Verifier note: Manual verify: [la luna] af1000d; all 4 interpreter_builtin_values tests passed. Restoring marker-string registration and dispatch made forged_builtin_string_is_not_callable, higher_order_builtins_have_the_same_kind_as_other_builtins, and higher_order_builtins_print_like_other_builtins fail: forged string called gaun, kind was string, marker printed verbatim. Source restored clean.

## Notes

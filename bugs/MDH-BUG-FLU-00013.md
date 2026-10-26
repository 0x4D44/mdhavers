# MDH-BUG-FLU-00013 — Higher-order builtins are callable magic strings

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T224201Z-df00531b
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00013-run-verify-20261026T224201Z-df00531b
- **Owner base:** 5122a4de097a3cb949560c57a78e74948858cacb
- **Owner fingerprint:** sha256:0a89b0ddd3c6c6446c3d5acb43ed4b30926674f2e14a6226b11350d289b39159
- **Owner since:** 2026-10-26T22:42:01Z
- **Owner until:** 2026-10-27T02:42:01Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T04:03:54Z, deltic:auto role=fix run=fix-20261023T040348Z-1113e70e branch=task/bug-MDH-BUG-FLU-00013-run-fix-20261023T040348Z-1113e70e code=af1000d gate=manual)

## Observation

Confirmed: whit_kind(gaun) is "string", blether gaun prints __builtin_gaun__, and any string with that prefix is callable (interpreter.rs ~7849, ~12238).

## Fix

<unfixed — raised only>

## Notes

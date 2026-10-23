# MDH-BUG-FLU-00042 — JS f-string text does not escape backslashes

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T021721Z-4a4d920c
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00042-run-fix-20261023T021721Z-4a4d920c
- **Owner base:** e1fc8e304ca05534d8156c1177aa33f571783d70
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T02:17:21Z
- **Owner until:** 2026-10-23T06:17:21Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed under node: "C:\\users" in an f-string gives a SyntaxError / changed text (compiler.rs:1265-1278).

## Fix

<unfixed — raised only>

## Notes

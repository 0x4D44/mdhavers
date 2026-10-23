# MDH-BUG-FLU-00075 — Newline inside call arguments or before a leading-dot method chain is a parse error

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-23T02:06:58Z
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
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:15:41Z, deltic:auto role=fix run=fix-20261023T031535Z-e74ec7fe branch=task/bug-MDH-BUG-FLU-00075-run-fix-20261023T031535Z-e74ec7fe code=e5fbb29 gate=manual)

## Observation

Found by the showcase lane: splitting a long call's arguments across lines, or starting a continuation line with `.method()`, fails to parse in both backends.

## Fix

<unfixed — raised only>

## Notes

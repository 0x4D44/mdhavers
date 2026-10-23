# MDH-BUG-FLU-00075 — Newline inside call arguments or before a leading-dot method chain is a parse error

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-23T02:06:58Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T031535Z-e74ec7fe
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00075-run-fix-20261023T031535Z-e74ec7fe
- **Owner base:** e42d8f78be442ff4ba94d01f093b6daa5b77a8fc
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:15:35Z
- **Owner until:** 2026-10-23T07:15:35Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the showcase lane: splitting a long call's arguments across lines, or starting a continuation line with `.method()`, fails to parse in both backends.

## Fix

<unfixed — raised only>

## Notes

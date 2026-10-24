# MDH-BUG-FLU-00095 — Native: reading a forward global before its ken has run gives naething instead of an error

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T21:37:15Z
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
- **State history:** Open (2026-10-23T21:37:15Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T01:42:59Z, deltic:auto role=fix run=fix-20261024T014254Z-5710c3e3 branch=task/bug-MDH-BUG-FLU-00095-run-fix-20261024T014254Z-5710c3e3 code=5cf1a3f gate=manual)

## Observation

Found by lane SEM-N 2026-10-23 after MDH-BUG-FLU-00072 (forward globals). dae f() { gie later }  blether f()  ken later = 1 : the interpreter raises an undefined-variable error at the call; native prints naething because the global slot exists but is still nil.

Evidence fingerprint: `manual:v1:native-reading-a-forward-global-before-its-ken--e4663332c38d42ae`


## Fix

<unfixed — raised only>

## Notes

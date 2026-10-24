# MDH-BUG-FLU-00095 — Native: reading a forward global before its ken has run gives naething instead of an error

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T21:37:15Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T014254Z-5710c3e3
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00095-run-fix-20261024T014254Z-5710c3e3
- **Owner base:** 5cf1a3f4a283d1bda1eebde9d9239d9a71d6b27f
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T01:42:54Z
- **Owner until:** 2026-10-24T05:42:54Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T21:37:15Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane SEM-N 2026-10-23 after MDH-BUG-FLU-00072 (forward globals). dae f() { gie later }  blether f()  ken later = 1 : the interpreter raises an undefined-variable error at the call; native prints naething because the global slot exists but is still nil.

Evidence fingerprint: `manual:v1:native-reading-a-forward-global-before-its-ken--e4663332c38d42ae`


## Fix

<unfixed — raised only>

## Notes

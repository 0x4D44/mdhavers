# MDH-BUG-FLU-00008 — bytes_append(b, b) panics with RefCell already borrowed

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
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
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:49:19Z, deltic:auto role=fix run=fix-20261023T014914Z-c57ec52b branch=task/bug-MDH-BUG-FLU-00008-run-fix-20261023T014914Z-c57ec52b code=d84823a gate=manual)

## Observation

Confirmed: panics at interpreter.rs ~2760; other.borrow() held while borrow_mut() on the same Rc.

## Fix

<unfixed — raised only>

## Notes

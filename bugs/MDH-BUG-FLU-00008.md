# MDH-BUG-FLU-00008 — bytes_append(b, b) panics with RefCell already borrowed

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T014914Z-c57ec52b
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00008-run-fix-20261023T014914Z-c57ec52b
- **Owner base:** 3c42dff416d5aca3568569f9f8d24fc9abe50cb7
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:49:14Z
- **Owner until:** 2026-10-23T05:49:14Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: panics at interpreter.rs ~2760; other.borrow() held while borrow_mut() on the same Rc.

## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00036 — Log filter rule `net` also matches target `network`

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** logging
- **Raised:** 2026-10-22T23:24:09Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T024915Z-2f4d925d
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00036-run-fix-20261023T024915Z-2f4d925d
- **Owner base:** 5dc87c0cc7d4b38ce76affe47fdc9d478e68f686
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T02:49:15Z
- **Owner until:** 2026-10-23T06:49:15Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: logging.rs:38 uses a plain starts_with with no segment boundary.

## Fix

<unfixed — raised only>

## Notes

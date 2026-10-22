# MDH-BUG-FLU-00036 — Log filter rule `net` also matches target `network`

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** logging
- **Raised:** 2026-10-22T23:24:09Z
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
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: logging.rs:38 uses a plain starts_with with no segment boundary.

## Fix

<unfixed — raised only>

## Notes

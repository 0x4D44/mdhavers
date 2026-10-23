# MDH-BUG-FLU-00086 — Native runtime errors lack the interpreter's " at line N" location

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T03:52:56Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T043052Z-0316ab2e
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00086-run-fix-20261023T043052Z-0316ab2e
- **Owner base:** 8cc5241ded54e0c024b17fba4d2b1ec3c3305b8d
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T04:30:52Z
- **Owner until:** 2026-10-23T08:30:52Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T03:52:56Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the native follow-ups lane 2026-10-23: e.g. native divide-by-zero prints the Scots message without " at line N", while the interpreter includes the line. Caught hurls now carry "Hurled at line N:" (MDH-BUG-FLU-00073), but runtime-raised errors do not.

## Fix

<unfixed — raised only>

## Notes

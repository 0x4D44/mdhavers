# MDH-BUG-FLU-00086 — Native runtime errors lack the interpreter's " at line N" location

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T03:52:56Z
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
- **State history:** Open (2026-10-23T03:52:56Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the native follow-ups lane 2026-10-23: e.g. native divide-by-zero prints the Scots message without " at line N", while the interpreter includes the line. Caught hurls now carry "Hurled at line N:" (MDH-BUG-FLU-00073), but runtime-raised errors do not.

## Fix

<unfixed — raised only>

## Notes

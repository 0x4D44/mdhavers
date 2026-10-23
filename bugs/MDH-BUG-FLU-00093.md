# MDH-BUG-FLU-00093 — Native integer overflow error lacks the interpreter's line number

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T04:31:22Z
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
- **State history:** Open (2026-10-23T04:31:22Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the native hardening lane 2026-10-23: the interpreter says "too muckle at line N - it's overflowed!" (line mid-message); native overflow errors carry no line. Other native runtime type errors ("Och! Type error in 'x'") also still differ from the interpreter's wording.

## Fix

<unfixed — raised only>

## Notes

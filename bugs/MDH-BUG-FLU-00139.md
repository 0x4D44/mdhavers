# MDH-BUG-FLU-00139 — Native tak/drap reject strings that the interpreter accepts

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T06:49:39Z
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
- **State history:** Open (2026-10-25T06:49:39Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane N8 2026-10-25 (executed) in an examples sweep: examples/builtins/list_functions.braw exits 1 natively at tak('Hello World', 5) with "Och! Type error in 'tak': got string"; the interpreter accepts strings (first/last n characters) and exits 0. drap likely fails the same way. Fix the native inline/runtime tak and drap to handle strings like the interpreter.

Evidence fingerprint: `manual:v1:native-tak-drap-reject-strings-that-the-interpreter-accepts`


## Fix

<unfixed — raised only>

## Notes

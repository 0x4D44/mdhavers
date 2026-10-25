# MDH-BUG-FLU-00141 — Native: repeat_char undefined in examples/stdlib/tui_demo.braw

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T06:49:48Z
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
- **State history:** Open (2026-10-25T06:49:48Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane N8 2026-10-25 (executed): examples/stdlib/tui_demo.braw exits 1 natively with "'repeat_char' hasnae been defined yet at line 370"; the interpreter exits 0. Find where repeat_char comes from (stdlib module import or prelude) and why native does not see it.

Evidence fingerprint: `manual:v1:native-repeat-char-undefined-in-examples-stdlib-tui-demo-braw`


## Fix

<unfixed — raised only>

## Notes

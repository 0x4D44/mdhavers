# MDH-BUG-FLU-00140 — Native backside_forrit rejects a list that the interpreter reverses

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T06:49:44Z
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
- **State history:** Open (2026-10-25T06:49:44Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane N8 2026-10-25 (executed): examples/builtins/prelude_demo.braw exits 1 natively at backside_forrit(numbers) with "Type error in 'reverse_str': got list"; the interpreter (prelude/builtin) accepts lists and exits 0. Native maps backside_forrit to a string-only reverse.

Evidence fingerprint: `manual:v1:native-backside-forrit-rejects-a-list-that-the--bae3094a14b6346d`


## Fix

<unfixed — raised only>

## Notes

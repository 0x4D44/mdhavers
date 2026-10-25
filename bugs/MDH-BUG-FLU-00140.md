# MDH-BUG-FLU-00140 — Native backside_forrit rejects a list that the interpreter reverses

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T06:49:44Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261025T065500Z-77cde6ad
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00140-run-fix-20261025T065500Z-77cde6ad
- **Owner base:** fca2255d8c3c7f66146289d70d13ec48622f0de1
- **Owner fingerprint:** -
- **Owner since:** 2026-10-25T06:55:00Z
- **Owner until:** 2026-10-25T10:55:00Z
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

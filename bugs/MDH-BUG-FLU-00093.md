# MDH-BUG-FLU-00093 — Native integer overflow error lacks the interpreter's line number

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T04:31:22Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T014305Z-ccbcbaf8
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00093-run-fix-20261024T014305Z-ccbcbaf8
- **Owner base:** 9edbbbf7f22380531abef02fe11037371843404a
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T01:43:05Z
- **Owner until:** 2026-10-24T05:43:05Z
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

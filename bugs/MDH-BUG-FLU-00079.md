# MDH-BUG-FLU-00079 — Deeply nested acyclic list equality and long import chains are not covered by the stack guard

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** interpreter
- **Raised:** 2026-10-23T02:06:58Z
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
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the interpreter lane: list == recursion on very deep (non-cyclic) nesting has no depth bound, and module import chains recurse through fetch without the call-depth/stack guard.

## Fix

<unfixed — raised only>

## Notes

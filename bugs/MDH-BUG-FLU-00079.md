# MDH-BUG-FLU-00079 — Deeply nested acyclic list equality and long import chains are not covered by the stack guard

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** interpreter
- **Raised:** 2026-10-23T02:06:58Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T032322Z-8648a049
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00079-run-fix-20261023T032322Z-8648a049
- **Owner base:** e60de69fb3fa62ae22df2fd923ebbb399ae945ac
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:23:22Z
- **Owner until:** 2026-10-23T07:23:22Z
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

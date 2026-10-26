# MDH-BUG-FLU-00079 — Deeply nested acyclic list equality and long import chains are not covered by the stack guard

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** interpreter
- **Raised:** 2026-10-23T02:06:58Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T224201Z-f73a8c2c
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00079-run-verify-20261026T224201Z-f73a8c2c
- **Owner base:** 5122a4de097a3cb949560c57a78e74948858cacb
- **Owner fingerprint:** sha256:88f7dfb5606df9a0ab573c297280eab74f126f459960beeabe8374202b783482
- **Owner since:** 2026-10-26T22:42:01Z
- **Owner until:** 2026-10-27T02:42:01Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:23:28Z, deltic:auto role=fix run=fix-20261023T032322Z-8648a049 branch=task/bug-MDH-BUG-FLU-00079-run-fix-20261023T032322Z-8648a049 code=a920344 gate=manual)

## Observation

Found by the interpreter lane: list == recursion on very deep (non-cyclic) nesting has no depth bound, and module import chains recurse through fetch without the call-depth/stack guard.

## Fix

<unfixed — raised only>

## Notes

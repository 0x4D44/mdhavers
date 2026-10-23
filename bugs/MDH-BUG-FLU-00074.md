# MDH-BUG-FLU-00074 — Native float printing differs from the interpreter (0.1 + 0.2)

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
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
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:51:25Z, deltic:auto role=fix run=fix-20261023T035119Z-10cb8c65 branch=task/bug-MDH-BUG-FLU-00074-run-fix-20261023T035119Z-10cb8c65 code=6029d44 gate=manual)

## Observation

Found by the showcase lane: interpreter prints 0.30000000000000004, native prints 0.3 (shortest-roundtrip vs %g-style formatting).

## Fix

<unfixed — raised only>

## Notes

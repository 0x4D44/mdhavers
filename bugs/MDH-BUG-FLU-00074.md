# MDH-BUG-FLU-00074 — Native float printing differs from the interpreter (0.1 + 0.2)

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T02:06:58Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T035119Z-10cb8c65
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00074-run-fix-20261023T035119Z-10cb8c65
- **Owner base:** ce765a0d433ade30185226ae5e2e24cc33d61d50
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:51:19Z
- **Owner until:** 2026-10-23T07:51:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the showcase lane: interpreter prints 0.30000000000000004, native prints 0.3 (shortest-roundtrip vs %g-style formatting).

## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00071 — Native: a user function named main breaks lookups of other functions

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-23T02:06:57Z
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
- **State history:** Open (2026-10-23T02:06:57Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:51:02Z, deltic:auto role=fix run=fix-20261023T035055Z-62298b79 branch=task/bug-MDH-BUG-FLU-00071-run-fix-20261023T035055Z-62298b79 code=6029d44 gate=manual)

## Observation

Found by the showcase lane: `dae helper() { gie 1 }` / `dae main() { blether helper() }` / `main()` -> "'helper' hasnae been defined yet" natively.

## Fix

<unfixed — raised only>

## Notes

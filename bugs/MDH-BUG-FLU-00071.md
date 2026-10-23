# MDH-BUG-FLU-00071 — Native: a user function named main breaks lookups of other functions

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-23T02:06:57Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T035055Z-62298b79
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00071-run-fix-20261023T035055Z-62298b79
- **Owner base:** 8a6829261a996bee551fc4823a395e5b14186d08
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:50:55Z
- **Owner until:** 2026-10-23T07:50:55Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:57Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the showcase lane: `dae helper() { gie 1 }` / `dae main() { blether helper() }` / `main()` -> "'helper' hasnae been defined yet" natively.

## Fix

<unfixed — raised only>

## Notes

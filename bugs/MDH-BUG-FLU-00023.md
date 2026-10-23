# MDH-BUG-FLU-00023 — Compound assignment rejected on fields and indexes

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-22T23:24:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T015145Z-b25ea6a2
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00023-run-fix-20261023T015145Z-b25ea6a2
- **Owner base:** e492fa85c073c665029dae502aff458e0f8ba37b
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:51:45Z
- **Owner until:** 2026-10-23T05:51:45Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: masel.n += 1 and xs[0] += 1 -> "Invalid compound assignment target" (parser.rs:704-717).

## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00023 — Compound assignment rejected on fields and indexes

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-22T23:24:08Z
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
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:51:50Z, deltic:auto role=fix run=fix-20261023T015145Z-b25ea6a2 branch=task/bug-MDH-BUG-FLU-00023-run-fix-20261023T015145Z-b25ea6a2 code=8204122 gate=manual)

## Observation

Confirmed: masel.n += 1 and xs[0] += 1 -> "Invalid compound assignment target" (parser.rs:704-717).

## Fix

<unfixed — raised only>

## Notes

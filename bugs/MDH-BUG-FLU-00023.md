# MDH-BUG-FLU-00023 — Compound assignment rejected on fields and indexes

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-22T23:24:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T233955Z-f74beac8
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00023-run-verify-20261026T233955Z-f74beac8
- **Owner base:** 3a7321efd6af1e16d34dcd7fed8e7df5ad38c691
- **Owner fingerprint:** sha256:9154263e0caa2bc79e42cf57796bbadebb7590c197af41de74c930cf8dd0964f
- **Owner since:** 2026-10-26T23:39:55Z
- **Owner until:** 2026-10-27T03:39:55Z
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

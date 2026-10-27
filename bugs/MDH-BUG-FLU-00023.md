# MDH-BUG-FLU-00023 — Compound assignment rejected on fields and indexes

- **State:** Closed
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
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:51:50Z, deltic:auto role=fix run=fix-20261023T015145Z-b25ea6a2 branch=task/bug-MDH-BUG-FLU-00023-run-fix-20261023T015145Z-b25ea6a2 code=8204122 gate=manual) -> Closed (2026-10-27T01:51:48Z, deltic:manual role=verify run=verify-20261026T233955Z-f74beac8 verified_fix_run=fix-20261023T015145Z-b25ea6a2 verdict=close)

## Observation

Confirmed: masel.n += 1 and xs[0] += 1 -> "Invalid compound assignment target" (parser.rs:704-717).

## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:51:48Z, deltic:manual run=verify-20261026T233955Z-f74beac8 verified_fix_run=fix-20261023T015145Z-b25ea6a2 verdict=close)

Verifier note: Manual verify: [lc luna] 8204122: compound_assignment_on_fields_and_indexes passed; disabling the Get/Index lowering branches reproduced `That compound assignment target's ower complicated` at `masel.n += 1`. Source diff restored clean.

## Notes

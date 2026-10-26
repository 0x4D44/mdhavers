# MDH-BUG-FLU-00104 — JS runtime has no creel (set) builtins

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-24T02:09:56Z
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
- **State history:** Open (2026-10-24T02:09:56Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T02:25:20Z, deltic:auto role=fix run=fix-20261024T022514Z-b664f533 branch=task/bug-MDH-BUG-FLU-00104-run-fix-20261024T022514Z-b664f533 code=4f64269 gate=manual) -> Closed (2026-10-26T23:39:42Z, deltic:manual role=verify run=verify-20261026T225713Z-40da5ec1 verified_fix_run=fix-20261024T022514Z-b664f533 verdict=close)

## Observation

Found by lane X1 2026-10-24 (and SEM-I allowlist notes): compiling a program that calls creel(...) to JS fails with ReferenceError: creel is not defined. Several goldens are allowlisted in tests/js_parity_allowlist.txt for this, incl. showcase_bothy_adventure and creel_structural_equality. Needs creel construction, membership, add/remove and set ops with interpreter display and structural equality.

Evidence fingerprint: `manual:v1:js-runtime-has-no-creel-set-builtins`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:39:42Z, deltic:manual run=verify-20261026T225713Z-40da5ec1 verified_fix_run=fix-20261024T022514Z-b664f533 verdict=close)

Verifier note: Manual verify: [lb luna] 4f64269 (implementation 578487c); parity_js_creel_builtins_mdh_bug_flu_00104 passed construction, set ops, equality and aliasing cases; removing creel registration failed all 3 parity assertions with `creel is not a function`.

## Notes

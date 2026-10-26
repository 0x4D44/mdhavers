# MDH-BUG-FLU-00104 — JS runtime has no creel (set) builtins

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-24T02:09:56Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T225713Z-40da5ec1
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00104-run-verify-20261026T225713Z-40da5ec1
- **Owner base:** 6cca1b9724e089b14af0018739faf304b5548a5a
- **Owner fingerprint:** sha256:c4930c1bd6c249f3f8180a3be65531434a81dc246c1aed5416a3a05e944da4fb
- **Owner since:** 2026-10-26T22:57:13Z
- **Owner until:** 2026-10-27T02:57:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T02:09:56Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T02:25:20Z, deltic:auto role=fix run=fix-20261024T022514Z-b664f533 branch=task/bug-MDH-BUG-FLU-00104-run-fix-20261024T022514Z-b664f533 code=4f64269 gate=manual)

## Observation

Found by lane X1 2026-10-24 (and SEM-I allowlist notes): compiling a program that calls creel(...) to JS fails with ReferenceError: creel is not defined. Several goldens are allowlisted in tests/js_parity_allowlist.txt for this, incl. showcase_bothy_adventure and creel_structural_equality. Needs creel construction, membership, add/remove and set ops with interpreter display and structural equality.

Evidence fingerprint: `manual:v1:js-runtime-has-no-creel-set-builtins`


## Fix

<unfixed — raised only>

## Notes

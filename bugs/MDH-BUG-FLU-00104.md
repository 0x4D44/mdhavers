# MDH-BUG-FLU-00104 — JS runtime has no creel (set) builtins

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-24T02:09:56Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T022514Z-b664f533
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00104-run-fix-20261024T022514Z-b664f533
- **Owner base:** 4f6426940a0f13c9785739a0ac03c858239c4582
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T02:25:14Z
- **Owner until:** 2026-10-24T06:25:14Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T02:09:56Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane X1 2026-10-24 (and SEM-I allowlist notes): compiling a program that calls creel(...) to JS fails with ReferenceError: creel is not defined. Several goldens are allowlisted in tests/js_parity_allowlist.txt for this, incl. showcase_bothy_adventure and creel_structural_equality. Needs creel construction, membership, add/remove and set ops with interpreter display and structural equality.

Evidence fingerprint: `manual:v1:js-runtime-has-no-creel-set-builtins`


## Fix

<unfixed — raised only>

## Notes

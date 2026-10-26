# MDH-BUG-FLU-00036 — Log filter rule `net` also matches target `network`

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** logging
- **Raised:** 2026-10-22T23:24:09Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T225713Z-7b796838
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00036-run-verify-20261026T225713Z-7b796838
- **Owner base:** 6cca1b9724e089b14af0018739faf304b5548a5a
- **Owner fingerprint:** sha256:e662e372eda02e4959ea14668adc47a6eca57270df08a733ff3edac1ed067d88
- **Owner since:** 2026-10-26T22:57:13Z
- **Owner until:** 2026-10-27T02:57:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:49:20Z, deltic:auto role=fix run=fix-20261023T024915Z-2f4d925d branch=task/bug-MDH-BUG-FLU-00036-run-fix-20261023T024915Z-2f4d925d code=77f950a gate=manual)

## Observation

Confirmed: logging.rs:38 uses a plain starts_with with no segment boundary.

## Fix

<unfixed — raised only>

## Notes

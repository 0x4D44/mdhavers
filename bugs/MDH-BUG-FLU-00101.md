# MDH-BUG-FLU-00101 — JS runtime lacks snooze and get_key used by the tri showcases

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-23T22:36:18Z
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
- **State history:** Open (2026-10-23T22:36:18Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T01:49:01Z, deltic:auto role=fix run=fix-20261024T014856Z-6e1aa3b0 branch=task/bug-MDH-BUG-FLU-00101-run-fix-20261024T014856Z-6e1aa3b0 code=e3865f0 gate=manual)

## Observation

Found by lane J2 2026-10-23 by reading: examples/showcases/tri_showcase/tri_showcase_controls.braw calls snooze and get_key, which the JS runtime does not define, so the showcase cannot run under the JS backend.

Evidence fingerprint: `manual:v1:js-runtime-lacks-snooze-and-get-key-used-by-the-tri-showcases`


## Fix

<unfixed — raised only>

## Notes

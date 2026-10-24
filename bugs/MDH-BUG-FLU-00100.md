# MDH-BUG-FLU-00100 — JS runtime lacks trig builtins (cos, sin) used by the tri showcases

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-23T22:22:18Z
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
- **State history:** Open (2026-10-23T22:22:18Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T01:38:37Z, deltic:auto role=fix run=fix-20261024T013831Z-411063e5 branch=task/bug-MDH-BUG-FLU-00100-run-fix-20261024T013831Z-411063e5 code=50f0079 gate=manual)

## Observation

Found by lane C 2026-10-23 by reading (not run): the tri showcase programs call cos/sin, which runtime/js/core_runtime.js and the emitted runtime do not define, so they likely fail under the JS backend. Verify by compiling examples/showcases/tri_showcase*.braw to JS and running in node.

Evidence fingerprint: `manual:v1:js-runtime-lacks-trig-builtins-cos-sin-used-by-the-tri-showcases`


## Fix

<unfixed — raised only>

## Notes

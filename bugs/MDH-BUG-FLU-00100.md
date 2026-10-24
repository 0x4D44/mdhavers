# MDH-BUG-FLU-00100 — JS runtime lacks trig builtins (cos, sin) used by the tri showcases

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-23T22:22:18Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T013831Z-411063e5
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00100-run-fix-20261024T013831Z-411063e5
- **Owner base:** 8f495c1cf1ccbee1b67973d77485108ed2883b27
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T01:38:31Z
- **Owner until:** 2026-10-24T05:38:31Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T22:22:18Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane C 2026-10-23 by reading (not run): the tri showcase programs call cos/sin, which runtime/js/core_runtime.js and the emitted runtime do not define, so they likely fail under the JS backend. Verify by compiling examples/showcases/tri_showcase*.braw to JS and running in node.

Evidence fingerprint: `manual:v1:js-runtime-lacks-trig-builtins-cos-sin-used-by-the-tri-showcases`


## Fix

<unfixed — raised only>

## Notes

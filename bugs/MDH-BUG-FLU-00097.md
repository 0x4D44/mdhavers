# MDH-BUG-FLU-00097 — JS: three.js property writes and method calls receive BigInt integers after F5

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-23T22:22:04Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T013808Z-43828961
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00097-run-fix-20261024T013808Z-43828961
- **Owner base:** 50f0079ce695b43b9c33a2612c0a96f0b24662dd
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T01:38:08Z
- **Owner until:** 2026-10-24T05:38:08Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T22:22:04Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane C (F5 BigInt ints, 7fbcda1) 2026-10-23. Only tri constructors convert args via nums(); property writes like sun.position.x = 2 (48 sites across examples/features/graphics3d_basics.braw and the three tri_showcase files) and calls like kamera.luik_at(0, 0, 0) pass BigInt to three.js, which expects numbers. Fix: route tri property sets and method calls through a converting helper.

Evidence fingerprint: `manual:v1:js-three-js-property-writes-and-method-calls-re-bf347b67f9f35ac5`


## Fix

<unfixed — raised only>

## Notes

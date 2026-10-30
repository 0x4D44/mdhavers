# MDH-BUG-FLU-00250 — bogle: body_substep picks the wrong neighbour when a sector has two portals on one straight line

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** bogle
- **Raised:** 2026-10-30T13:16:32Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261030T141042Z-e6b97de8
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00250-run-fix-20261030T141042Z-e6b97de8
- **Owner base:** 15dce43184f7cbcf6d8fb8713cec62f1b13c5a4c
- **Owner fingerprint:** -
- **Owner since:** 2026-10-30T14:10:42Z
- **Owner until:** 2026-10-30T18:10:42Z
- **Owner session:** flux/2a9090d7-08f7-48c7-bee8-7b7d1d963ab2 (run fix-20261030T141042Z-e6b97de8)
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-30T13:16:32Z, raised via `deltic bugs new`)

## Observation

Reported by the E1M2 build worker, 2026-10-30, on trunk 0.3.140-era engine (not yet independently reproduced by the lead): engine/move.braw:body_substep decides the sector a body crosses into by testing each wall's infinite line rather than the wall segment. In a draft E1M2 layout where one sector had two portal walls collinear along the same straight edge, the player was assigned the wrong neighbouring sector after crossing one of them, and then stopped collecting pickups (the silver key and bagpipes were not picked up on the autopilot route). bogle_trace reportedly uses the same whole-line test. Expected: crossing a portal assigns the sector behind that portal segment. The shipped E1M2 builder rejects such layouts, so E1M2 does not trigger it; hand-made maps can.

## Fix

<unfixed — raised only>

## Notes

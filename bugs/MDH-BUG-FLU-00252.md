# MDH-BUG-FLU-00252 — bogle: a gamekeeper gets stuck when a corner of wall is between him and the player

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** bogle
- **Raised:** 2026-10-30T15:47:56Z
- **Discovery source:** Human
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261030T154851Z-ee286075
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00252-run-fix-20261030T154851Z-ee286075
- **Owner base:** ffd8fc29ccaaa6f250cbb6c3d40bd4aa2a96a212
- **Owner fingerprint:** -
- **Owner since:** 2026-10-30T15:48:51Z
- **Owner until:** 2026-10-30T19:48:51Z
- **Owner session:** flux/2a9090d7-08f7-48c7-bee8-7b7d1d963ab2 (run fix-20261030T154851Z-ee286075)
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-30T15:47:56Z, raised via `deltic bugs new`)

## Observation

Observed by Arthur playtesting on 0.3.147-era native build, 2026-10-30: 'If there's a bit of wall (say a corner) between a gamekeeper and me, then gamekeeper gets stuck.' Expected: a keeper that can see the player but not reach him in a straight line works its way round the obstruction (Doom-style chase direction) rather than stalling. Actual: the keeper stays stuck at the corner. Likely mechanism (unconfirmed): engine/ai.braw kind 2 only charges in a straight line once bogle_sight succeeds; a charge into wall ends in a 35-tic stun, then patrol, then sight, windup and the same blocked charge.

## Fix

<unfixed — raised only>

## Notes

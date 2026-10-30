# MDH-BUG-FLU-00249 — bogle: gamekeeper falls off a high ledge in E1M2 and ends up stuck inside the wall below

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** bogle
- **Raised:** 2026-10-30T13:16:24Z
- **Discovery source:** Human
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
- **State history:** Open (2026-10-30T13:16:24Z, raised via `deltic bugs new`) -> Fixed (2026-10-30T14:32:46Z, deltic:auto role=fix run=fix-20261030T141050Z-8d35cb75 branch=task/bug-MDH-BUG-FLU-00249-run-fix-20261030T141050Z-8d35cb75 code=8a41717a262bdddc571ebfb9a3c739bfb15e0077 gate=manual)

## Observation

Observed by Arthur playtesting E1M2 (The Distillery) on 0.3.143, native build, 2026-10-30: 'a groundskeeper fell off a higher bit of wall in one of the rooms in E1M2 and got stuck in the wall below.' Expected: walking enemies do not step off drops taller than a step (Doom monsters refuse drops over 24 units), and an actor never ends inside solid wall. Actual: keeper dropped from a raised area and was left embedded in the wall underneath. Room not identified. Actors move via engine/things.braw:actor_move -> move_body, which snaps z to the floor of whatever sector body_substep assigns; no drop limit is applied to actors.

## Fix

<unfixed — raised only>

## Notes

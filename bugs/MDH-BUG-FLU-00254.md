# MDH-BUG-FLU-00254 — bogle_parity e1m2 playthrough fails: e1m2.hashes has 39 rows, test expects 38

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** tests
- **Raised:** 2026-11-01T18:48:46Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261101T223822Z-b3718553
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00254-run-fix-20261101T223822Z-b3718553
- **Owner base:** 8b868bbdcab440a7e57a3bc4baee2c95843960ae
- **Owner fingerprint:** -
- **Owner since:** 2026-11-01T22:38:22Z
- **Owner until:** 2026-11-02T02:38:22Z
- **Owner session:** flux/2a9090d7-08f7-48c7-bee8-7b7d1d963ab2 (run fix-20261101T223822Z-b3718553)
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-11-01T18:48:46Z, raised via `deltic bugs new --land`)

## Observation

Observed 2026-11-01 on a cold cargo test from origin/master edaf679 plus the story-campaign branch (tests/bogle_parity.rs and e1m2 files unchanged from trunk): e1m2_playthrough_matches_native_rc_modes_and_interpreter_prefix panicked at tests/bogle_parity.rs:313: 'e1m2.hashes must contain one row for each sampled frame, left: 39, right: 38'. be3e1e6 introduced check_playthrough("e1m2", 38) when games/bogle/replays/e1m2.hashes had 38 rows; a later trunk commit grew the file to 39 rows without updating the count. The 39 rows match native rc0/rc1 output (lead check). Expected: the test's row count matches the committed hashes.

Evidence fingerprint: `trunk-red:v2:mdhavers-bogle-parity-e1m2-playthrough-fails-e1-a1bcdc8b6faf3465`


## Fix

<unfixed — raised only>

## Notes

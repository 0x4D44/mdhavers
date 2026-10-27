# MDH-BUG-FLU-00092 — WASM host store never frees strings, lists or dicts

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-23T04:31:22Z
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
- **State history:** Open (2026-10-23T04:31:22Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T02:18:21Z, deltic:auto role=fix run=fix-20261024T021815Z-ec3adc88 branch=task/bug-MDH-BUG-FLU-00092-run-fix-20261024T021815Z-ec3adc88 code=c6b0df3 gate=manual) -> Closed (2026-10-27T01:32:14Z, deltic:manual role=verify run=verify-20261027T001425Z-50502a17 verified_fix_run=fix-20261024T021815Z-ec3adc88 verdict=close)

## Observation

Found by the WASM runner lane 2026-10-23: numbers are now inline or interned (MDH-BUG-FLU-00047), but every string, list and dict allocated by the wasm_runner host stays in the store for the life of the run, so long-running string-heavy programs grow without bound.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:32:14Z, deltic:manual run=verify-20261027T001425Z-50502a17 verified_fix_run=fix-20261024T021815Z-ec3adc88 verdict=close)

Verifier note: Manual verify: [ld luna] fix 2a619b1; long_string_building_loop_keeps_the_host_store_bounded passed with item 19999!; removing sweep release made it fail at the peak_store_size assertion (140004 vs <1000).

## Notes

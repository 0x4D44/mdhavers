# MDH-BUG-FLU-00202 — heave_oot ratio perf test flakes red on trunk: 33 ms baseline, 4x step, 10x limit

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** tests
- **Raised:** 2026-10-27T05:22:08Z
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
- **State history:** Open (2026-10-27T05:22:08Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-27T05:31:29Z, deltic:auto role=fix run=fix-20261027T052215Z-03955c31 branch=task/bug-MDH-BUG-FLU-00202-run-fix-20261027T052215Z-03955c31 code=05f257b gate=manual) -> Closed (2026-10-27T05:42:18Z, deltic:manual role=verify run=verify-20261027T053312Z-78fdbca5 verified_fix_run=fix-20261027T052215Z-03955c31 verdict=close)

## Observation

On trunk b2712ad (0.3.100), `cargo test --test interpreter_dict_remove_perf` fails `heave_oot_scales_linearly_not_quadratically`
(added f13e3f0 for MDH-BUG-FLU-00198), run alone at load average ~62:

    heave_oot looks quadratic: 4x the keys took over 330.74125ms (10x the 33.074125ms for 5k keys); linear is ~4x, quadratic ~16x

The independent verifier saw it fail 3/3 alone at 05:00 GMT; it passed in the lead's 04:10 check, so it is flaky, not
deterministic. Likely causes: a ~33 ms baseline is noise-sized; the 5k runs are on the main thread and the 20k run on a
spawned thread; a 4x size step leaves a thin margin between linear-with-cache-effects and quadratic (16x).

Evidence fingerprint: `trunk-red:v2:mdhavers-heave-oot-ratio-perf-test-flakes-red-o-191d4212d63fa7e6`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T05:42:18Z, deltic:manual run=verify-20261027T053312Z-78fdbca5 verified_fix_run=fix-20261027T052215Z-03955c31 verdict=close)

Verifier note: Manual verify: [lg luna] 05f257b; heave_oot_removal_costs_a_small_multiple_of_a_plain_loop passed 8/8 (removal/control ms: 170/136,61/122,89/141,150/135,281/94,159/43,110/53,55/118); 72144ea mutant (compact each delete) selected test failed ratio assertion at 215720/277 ms (>25x).

## Notes

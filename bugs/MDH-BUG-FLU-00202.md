# MDH-BUG-FLU-00202 — heave_oot ratio perf test flakes red on trunk: 33 ms baseline, 4x step, 10x limit

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** tests
- **Raised:** 2026-10-27T05:22:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261027T053312Z-78fdbca5
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00202-run-verify-20261027T053312Z-78fdbca5
- **Owner base:** 8ddd5f0c323e45b224a0dfb5d647908058dac8d5
- **Owner fingerprint:** sha256:d1079d7d8ed3598b2aafe914e33565c6bf5feff3936d6d1a75aab98ab42f6d31
- **Owner since:** 2026-10-27T05:33:12Z
- **Owner until:** 2026-10-27T09:33:12Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-27T05:22:08Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-27T05:31:29Z, deltic:auto role=fix run=fix-20261027T052215Z-03955c31 branch=task/bug-MDH-BUG-FLU-00202-run-fix-20261027T052215Z-03955c31 code=05f257b gate=manual)

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

## Notes

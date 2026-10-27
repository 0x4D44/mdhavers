# MDH-BUG-FLU-00198 — heave_oot_100k perf test fails on a loaded box: 30 s wall-clock limit for under 1 s of CPU

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** tests
- **Raised:** 2026-10-27T00:57:03Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261027T053312Z-f63fe685
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00198-run-verify-20261027T053312Z-f63fe685
- **Owner base:** 8ddd5f0c323e45b224a0dfb5d647908058dac8d5
- **Owner fingerprint:** sha256:ce4932ac1a8a3105df06e033cca1664ebac8b05bb2a35aa3dd9297b3ac45b587
- **Owner since:** 2026-10-27T05:33:12Z
- **Owner until:** 2026-10-27T09:33:12Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=1, indeterminate=0
- **State history:** Open (2026-10-27T00:57:03Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-27T04:51:23Z, deltic:auto role=fix run=fix-20261027T015330Z-7a888907 branch=task/bug-MDH-BUG-FLU-00198-run-fix-20261027T015330Z-7a888907 code=f13e3f0 gate=manual) -> Open (2026-10-27T05:22:54Z, deltic:manual role=verify run=verify-20261027T045159Z-b011d81c verified_fix_run=fix-20261027T015330Z-7a888907 verdict=doubt reason=reopen-regression-remains-red-under-load-restored-diff-clean) -> Fixed (2026-10-27T05:31:41Z, deltic:auto role=fix run=fix-20261027T053134Z-1ecc6be7 branch=task/bug-MDH-BUG-FLU-00198-run-fix-20261027T053134Z-1ecc6be7 code=05f257b gate=manual)

## Observation

`tests/interpreter_dict_remove_perf.rs::heave_oot_100k_keys_finishes_quickly` asserts a 30 s wall-clock limit.
On the shared FLUX Mac at load average 220-257 (2026-10-27 00:30-00:50 GMT) it failed three times on trunk-equivalent
code (S29 branch, no dict changes): 46.1 s, 40.9 s and 38.4 s. Under `/usr/bin/time -l` the 38.4 s run used 0.70 s
user CPU against 39.59 s real (6715 involuntary context switches): the work is well under a second of CPU, and the
time is spent waiting to be scheduled. A Luna verifier timed the same test at 3.04 s earlier that night.

Expected: the test catches a quadratic regression (100k removals) without failing on a loaded box.
Actual: fails whenever the box is busy, which blocks lead checks and drains on shared hosts.
Same class seen earlier: `llvm_hardening` dict timing tests and the WASM showcase timeouts (182, fixed by scaling).
Likely direction: measure the process's CPU time (getrusage) or compare against an in-process baseline, not wall time.

Evidence fingerprint: `manual:v1:heave-oot-100k-perf-test-fails-on-a-loaded-box--42aabce37fd3994b`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T05:22:54Z, deltic:manual run=verify-20261027T045159Z-b011d81c verified_fix_run=fix-20261027T015330Z-7a888907 verdict=doubt)

Verifier note: Manual verify: [lf luna] f13e3f0; heave_oot_scales_linearly_not_quadratically selected and failed 3/3 alone at its 10x timeout; pre-5432 100k/30s mutant passed 1/1 in 4.15s; current regression remains red under load; restored; diff clean

## Notes

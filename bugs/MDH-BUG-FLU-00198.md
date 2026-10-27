# MDH-BUG-FLU-00198 — heave_oot_100k perf test fails on a loaded box: 30 s wall-clock limit for under 1 s of CPU

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** tests
- **Raised:** 2026-10-27T00:57:03Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261027T015330Z-7a888907
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00198-run-fix-20261027T015330Z-7a888907
- **Owner base:** e865168d2677fc3dce5507be1446c7c214a8b7a8
- **Owner fingerprint:** -
- **Owner since:** 2026-10-27T01:53:30Z
- **Owner until:** 2026-10-27T05:53:30Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-27T00:57:03Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

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

## Notes

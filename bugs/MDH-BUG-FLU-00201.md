# MDH-BUG-FLU-00201 — dict_remove block-copy ratio test fails under parallel test load (27x vs 10x limit)

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** tests
- **Raised:** 2026-10-27T04:32:42Z
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
- **State history:** Open (2026-10-27T04:32:42Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-27T04:51:33Z, deltic:auto role=fix run=fix-20261027T043249Z-c729790a branch=task/bug-MDH-BUG-FLU-00201-run-fix-20261027T043249Z-c729790a code=65ae7f7 gate=manual) -> Closed (2026-10-27T05:23:43Z, deltic:manual role=verify run=verify-20261027T045159Z-86e7e129 verified_fix_run=fix-20261027T043249Z-c729790a verdict=close)

## Observation

tests/llvm_hardening.rs `dict_remove_copies_the_entries_in_blocks_instead_of_reinserting_them` (added b7b7836 for MDH-BUG-FLU-00091) failed once in a full parallel run of the native test set on trunk + test-only changes (2026-10-27 04:30 GMT, load average ~60, `--test-threads=6` alongside the other llvm_* targets):

    100 dict_remove calls took 517 ms against 19 ms for one rebuild

(limit: removes < 10 x rebuild). Run alone three times right after, it passed each time (0.59-1.13 s). The two timings are taken at different moments; the 100 removes are memcpy-bound (about 4.8 MB per call) and suffer more from memory-bandwidth contention with parallel tests than the rebuild does, so the ratio swings under concurrent load. The mutant it guards (re-insertion loop) measured 46x when written, so the margin to the limit is small under contention.

Expected: the test keeps catching the re-insertion mutant but does not fail in a normal parallel run.

Evidence fingerprint: `manual:v1:dict-remove-block-copy-ratio-test-fails-under-p-874d17f8d03db47c`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T05:23:43Z, deltic:manual run=verify-20261027T045159Z-86e7e129 verified_fix_run=fix-20261027T043249Z-c729790a verdict=close)

Verifier note: Manual verify: Independent Opus review (lead landed this): close - a load-flakiness fix cannot show a deterministic fails-before; adequate evidence is the paired, best-of-5 design plus the test still catching the re-insertion mutant (fixer-reported 24-27x vs 10x limit, not yet independently reproduced). Verifier: [lf luna] 65ae7f7; dict_remove_copies_the_entries_in_blocks_instead_of_reinserting_them passed 5/5 on trunk; reversing to prior unpaired timing stayed green 1/1, restored pass; test does not cover the fix

## Notes

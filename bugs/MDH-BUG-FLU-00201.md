# MDH-BUG-FLU-00201 — dict_remove block-copy ratio test fails under parallel test load (27x vs 10x limit)

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** tests
- **Raised:** 2026-10-27T04:32:42Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261027T045159Z-86e7e129
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00201-run-verify-20261027T045159Z-86e7e129
- **Owner base:** b2712adc7b304eb842407eae833911cd438de665
- **Owner fingerprint:** sha256:e5f2c09a6377fb7ddabe6b9aa87bcaf730b67edabf2cb3f095aa8f788acd5091
- **Owner since:** 2026-10-27T04:51:59Z
- **Owner until:** 2026-10-27T08:51:59Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-27T04:32:42Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-27T04:51:33Z, deltic:auto role=fix run=fix-20261027T043249Z-c729790a branch=task/bug-MDH-BUG-FLU-00201-run-fix-20261027T043249Z-c729790a code=65ae7f7 gate=manual)

## Observation

tests/llvm_hardening.rs `dict_remove_copies_the_entries_in_blocks_instead_of_reinserting_them` (added b7b7836 for MDH-BUG-FLU-00091) failed once in a full parallel run of the native test set on trunk + test-only changes (2026-10-27 04:30 GMT, load average ~60, `--test-threads=6` alongside the other llvm_* targets):

    100 dict_remove calls took 517 ms against 19 ms for one rebuild

(limit: removes < 10 x rebuild). Run alone three times right after, it passed each time (0.59-1.13 s). The two timings are taken at different moments; the 100 removes are memcpy-bound (about 4.8 MB per call) and suffer more from memory-bandwidth contention with parallel tests than the rebuild does, so the ratio swings under concurrent load. The mutant it guards (re-insertion loop) measured 46x when written, so the margin to the limit is small under contention.

Expected: the test keeps catching the re-insertion mutant but does not fail in a normal parallel run.

Evidence fingerprint: `manual:v1:dict-remove-block-copy-ratio-test-fails-under-p-874d17f8d03db47c`


## Fix

<unfixed — raised only>

## Notes

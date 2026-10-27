# MDH-BUG-FLU-00106 — A test writes a file named hello into the repo root and never removes it

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** tests
- **Raised:** 2026-10-24T02:14:14Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261027T045159Z-258d8026
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00106-run-verify-20261027T045159Z-258d8026
- **Owner base:** b2712adc7b304eb842407eae833911cd438de665
- **Owner fingerprint:** sha256:8c61c3749cb426bde6364d96807893de4f86692121565d7f6cb7d780211ba68e
- **Owner since:** 2026-10-27T04:51:59Z
- **Owner until:** 2026-10-27T08:51:59Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=1, indeterminate=0
- **State history:** Open (2026-10-24T02:14:14Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T02:58:41Z, deltic:auto role=fix run=fix-20261024T025834Z-392b3393 branch=task/bug-MDH-BUG-FLU-00106-run-fix-20261024T025834Z-392b3393 code=1c57be6 gate=manual) -> Open (2026-10-27T01:51:48Z, deltic:manual role=verify run=verify-20261026T233955Z-98f24b9c verified_fix_run=fix-20261024T025834Z-392b3393 verdict=doubt reason=reopen-lc-luna-47057d9-native-builtins-coverage-passed-on-trunk-without-a-root-h) -> Fixed (2026-10-27T04:50:50Z, deltic:auto role=fix run=fix-20261027T015303Z-9836d589 branch=task/bug-MDH-BUG-FLU-00106-run-fix-20261027T015303Z-9836d589 code=1f084a0 gate=manual)

## Observation

Seen twice by the lead (2026-10-23 R2 worktree, 2026-10-24 full-suite sweep on trunk b0bb4c7): after cargo test an untracked ASCII file 'hello' appears in the worktree root. Candidates that write "hello": tests/interpreter_builtin_values.rs, interpreter_misc_coverage.rs, interpreter_gap_coverage.rs, interpreter_native_builtins_coverage.rs, llvm_codegen_builtin_sweep_coverage.rs. Tests should write into a tempdir.

Evidence fingerprint: `manual:v1:a-test-writes-a-file-named-hello-into-the-repo--d99e084178303f80`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:51:48Z, deltic:manual run=verify-20261026T233955Z-98f24b9c verified_fix_run=fix-20261024T025834Z-392b3393 verdict=doubt)

Verifier note: Manual verify: [lc luna] 47057d9: native builtins coverage passed on trunk without a root `hello`; removing its temp-cwd setup left the same test green but created `.scratch/hello` (directory). Test does not cover the fix. Mutant/artifact restored.

## Notes

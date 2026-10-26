# MDH-BUG-FLU-00106 — A test writes a file named hello into the repo root and never removes it

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** tests
- **Raised:** 2026-10-24T02:14:14Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T233955Z-98f24b9c
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00106-run-verify-20261026T233955Z-98f24b9c
- **Owner base:** 3a7321efd6af1e16d34dcd7fed8e7df5ad38c691
- **Owner fingerprint:** sha256:8ddf7d3236b81e39882371e3bcb244b98f7953ecd7a8d88003aeb1e14fe2d922
- **Owner since:** 2026-10-26T23:39:55Z
- **Owner until:** 2026-10-27T03:39:55Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T02:14:14Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T02:58:41Z, deltic:auto role=fix run=fix-20261024T025834Z-392b3393 branch=task/bug-MDH-BUG-FLU-00106-run-fix-20261024T025834Z-392b3393 code=1c57be6 gate=manual)

## Observation

Seen twice by the lead (2026-10-23 R2 worktree, 2026-10-24 full-suite sweep on trunk b0bb4c7): after cargo test an untracked ASCII file 'hello' appears in the worktree root. Candidates that write "hello": tests/interpreter_builtin_values.rs, interpreter_misc_coverage.rs, interpreter_gap_coverage.rs, interpreter_native_builtins_coverage.rs, llvm_codegen_builtin_sweep_coverage.rs. Tests should write into a tempdir.

Evidence fingerprint: `manual:v1:a-test-writes-a-file-named-hello-into-the-repo--d99e084178303f80`


## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00106 — A test writes a file named hello into the repo root and never removes it

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** tests
- **Raised:** 2026-10-24T02:14:14Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T025834Z-392b3393
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00106-run-fix-20261024T025834Z-392b3393
- **Owner base:** b5218a9c4874883d73ad08bf1f042f262ff85e0f
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T02:58:34Z
- **Owner until:** 2026-10-24T06:58:34Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T02:14:14Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Seen twice by the lead (2026-10-23 R2 worktree, 2026-10-24 full-suite sweep on trunk b0bb4c7): after cargo test an untracked ASCII file 'hello' appears in the worktree root. Candidates that write "hello": tests/interpreter_builtin_values.rs, interpreter_misc_coverage.rs, interpreter_gap_coverage.rs, interpreter_native_builtins_coverage.rs, llvm_codegen_builtin_sweep_coverage.rs. Tests should write into a tempdir.

Evidence fingerprint: `manual:v1:a-test-writes-a-file-named-hello-into-the-repo--d99e084178303f80`


## Fix

<unfixed — raised only>

## Notes

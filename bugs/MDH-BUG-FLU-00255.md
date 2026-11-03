# MDH-BUG-FLU-00255 — Trunk red: ownership arity test fails on __mdh_screen_loop (no LAZY_DECLS call)

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** llvm-ownership
- **Raised:** 2026-11-02T06:17:22Z
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
- **State history:** Open (2026-11-02T06:17:22Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-11-02T06:19:31Z, deltic:auto role=fix run=fix-20261102T061730Z-d997f72a branch=task/bug-MDH-BUG-FLU-00255-run-fix-20261102T061730Z-d997f72a code=e2d604a587ca05a685dc13aea0123bfb6e21a1e7 gate=manual) -> Closed (2026-11-03T16:35:33Z, deltic:manual role=verify run=verify-20261103T162915Z-99e45fbf verified_fix_run=fix-20261102T061730Z-d997f72a verdict=close)

## Observation

Observed 2026-11-02 on clean origin/master 61380c0: 'cargo test --lib ownership' gives 4 passed, 1 failed: llvm::ownership::tests::every_declared_runtime_symbol_has_an_entry_with_its_arity, message '__mdh_screen_loop: entry for a symbol codegen does not declare'. RUNTIME_OWNERSHIP (src/llvm/ownership.rs:342) has a row for __mdh_screen_loop, but the test's LAZY_DECLS program never calls screen_loop, so codegen never declares it.

Evidence fingerprint: `trunk-red:v2:mdhavers-trunk-red-ownership-arity-test-fails-o-88c02b67059f1cc5`


## Fix

<unfixed — raised only>

### Verification summary (2026-11-03T16:35:33Z, deltic:manual run=verify-20261103T162915Z-99e45fbf verified_fix_run=fix-20261102T061730Z-d997f72a verdict=close)

Verifier note: Manual verify: Verified on origin/master 5cf079a (contains e2d604a): cargo test --lib ownership -> 5 passed, every_declared_runtime_symbol_has_an_entry_with_its_arity ok. Fails-before (method B, throwaway worktree): removed 'screen_loop(|t| t)' from LAZY_DECLS -> 4 passed 1 failed, panicked at src/llvm/ownership.rs:622 '__mdh_screen_loop: entry for a symbol codegen does not declare' (the recorded observation). Restored (git diff empty) -> 5 passed. Root cause (test program never declared the screen loop) addressed; RUNTIME_OWNERSHIP row is now exercised.

## Notes

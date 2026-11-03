# MDH-BUG-FLU-00255 — Trunk red: ownership arity test fails on __mdh_screen_loop (no LAZY_DECLS call)

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** llvm-ownership
- **Raised:** 2026-11-02T06:17:22Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261103T162915Z-99e45fbf
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00255-run-verify-20261103T162915Z-99e45fbf
- **Owner base:** 5cf079a745c3243c2deab6c64240b0dc2e35308f
- **Owner fingerprint:** sha256:5200b033a7e6d7cf42b2bbdb61e43c2f3165e47de625ed89711165b7e39f39f1
- **Owner since:** 2026-11-03T16:29:15Z
- **Owner until:** 2026-11-03T20:29:15Z
- **Owner session:** flux/2a9090d7-08f7-48c7-bee8-7b7d1d963ab2 (run verify-20261103T162915Z-99e45fbf)
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-11-02T06:17:22Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-11-02T06:19:31Z, deltic:auto role=fix run=fix-20261102T061730Z-d997f72a branch=task/bug-MDH-BUG-FLU-00255-run-fix-20261102T061730Z-d997f72a code=e2d604a587ca05a685dc13aea0123bfb6e21a1e7 gate=manual)

## Observation

Observed 2026-11-02 on clean origin/master 61380c0: 'cargo test --lib ownership' gives 4 passed, 1 failed: llvm::ownership::tests::every_declared_runtime_symbol_has_an_entry_with_its_arity, message '__mdh_screen_loop: entry for a symbol codegen does not declare'. RUNTIME_OWNERSHIP (src/llvm/ownership.rs:342) has a row for __mdh_screen_loop, but the test's LAZY_DECLS program never calls screen_loop, so codegen never declares it.

Evidence fingerprint: `trunk-red:v2:mdhavers-trunk-red-ownership-arity-test-fails-o-88c02b67059f1cc5`


## Fix

<unfixed — raised only>

## Notes

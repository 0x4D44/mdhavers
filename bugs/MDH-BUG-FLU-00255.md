# MDH-BUG-FLU-00255 — Trunk red: ownership arity test fails on __mdh_screen_loop (no LAZY_DECLS call)

- **State:** Open
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
- **State history:** Open (2026-11-02T06:17:22Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Observed 2026-11-02 on clean origin/master 61380c0: 'cargo test --lib ownership' gives 4 passed, 1 failed: llvm::ownership::tests::every_declared_runtime_symbol_has_an_entry_with_its_arity, message '__mdh_screen_loop: entry for a symbol codegen does not declare'. RUNTIME_OWNERSHIP (src/llvm/ownership.rs:342) has a row for __mdh_screen_loop, but the test's LAZY_DECLS program never calls screen_loop, so codegen never declares it.

Evidence fingerprint: `trunk-red:v2:mdhavers-trunk-red-ownership-arity-test-fails-o-88c02b67059f1cc5`


## Fix

<unfixed — raised only>

## Notes

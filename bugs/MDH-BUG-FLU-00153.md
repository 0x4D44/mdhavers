# MDH-BUG-FLU-00153 — Coverage-cfg lib unit tests fail to compile: LoopContext literals miss rc_scope_depth

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** llvm
- **Raised:** 2026-10-25T22:15:58Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261025T223241Z-49b8e1e9
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00153-run-fix-20261025T223241Z-49b8e1e9
- **Owner base:** 1c4b934f5362918e79fd63662d71b0bfcf870eed
- **Owner fingerprint:** -
- **Owner since:** 2026-10-25T22:32:41Z
- **Owner until:** 2026-10-26T02:32:41Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T22:15:58Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane U4 2026-10-25 (observed): RUSTFLAGS="--cfg coverage" cargo test --no-default-features --features cli,llvm,native --lib fails with E0063 missing field rc_scope_depth in LoopContext at src/llvm/codegen.rs:48066 and :48979 (coverage-only test code). The field was added by d5c9ff6 (R1 refcounting) and the cfg(coverage) tests were not updated, so make coverage / coverage-gate cannot build the lib tests. Fix: add rc_scope_depth: 0 (or the codegen's current depth) to both literals.

Evidence fingerprint: `manual:v1:coverage-cfg-lib-unit-tests-fail-to-compile-loo-bfbcebe3b8a2a693`


## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00153 — Coverage-cfg lib unit tests fail to compile: LoopContext literals miss rc_scope_depth

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** llvm
- **Raised:** 2026-10-25T22:15:58Z
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
- **State history:** Open (2026-10-25T22:15:58Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T23:21:11Z, deltic:auto role=fix run=fix-20261025T223241Z-49b8e1e9 branch=task/bug-MDH-BUG-FLU-00153-run-fix-20261025T223241Z-49b8e1e9 code=f5d5a24 gate=manual) -> Closed (2026-10-27T02:35:24Z, deltic:manual role=verify run=verify-20261027T013251Z-ee933361 verified_fix_run=fix-20261025T223241Z-49b8e1e9 verdict=close)

## Observation

Found by lane U4 2026-10-25 (observed): RUSTFLAGS="--cfg coverage" cargo test --no-default-features --features cli,llvm,native --lib fails with E0063 missing field rc_scope_depth in LoopContext at src/llvm/codegen.rs:48066 and :48979 (coverage-only test code). The field was added by d5c9ff6 (R1 refcounting) and the cfg(coverage) tests were not updated, so make coverage / coverage-gate cannot build the lib tests. Fix: add rc_scope_depth: 0 (or the codegen's current depth) to both literals.

Evidence fingerprint: `manual:v1:coverage-cfg-lib-unit-tests-fail-to-compile-loo-bfbcebe3b8a2a693`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T02:35:24Z, deltic:manual run=verify-20261027T013251Z-ee933361 verified_fix_run=fix-20261025T223241Z-49b8e1e9 verdict=close)

Verifier note: Manual verify: [le luna] f5d5a24; coverage --lib selected 1602 tests (1594 pass, 8 unrelated TLS-permission/arity failures); both literal tests passed 1/1. Removing both fields restores E0063, but this compile-only mutant cannot reach an assertion.

## Notes

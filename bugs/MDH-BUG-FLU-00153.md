# MDH-BUG-FLU-00153 — Coverage-cfg lib unit tests fail to compile: LoopContext literals miss rc_scope_depth

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** llvm
- **Raised:** 2026-10-25T22:15:58Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T194306Z-e26f5a90
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00153-run-verify-20261026T194306Z-e26f5a90
- **Owner base:** 36750bc8afd262a99873992d919d9ed250946aef
- **Owner fingerprint:** sha256:4ca106e2ff5140a83634939afda66381edbfd211776dffcaf0b506c43370a468
- **Owner since:** 2026-10-26T19:43:06Z
- **Owner until:** 2026-10-26T23:43:06Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T22:15:58Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T23:21:11Z, deltic:auto role=fix run=fix-20261025T223241Z-49b8e1e9 branch=task/bug-MDH-BUG-FLU-00153-run-fix-20261025T223241Z-49b8e1e9 code=f5d5a24 gate=manual)

## Observation

Found by lane U4 2026-10-25 (observed): RUSTFLAGS="--cfg coverage" cargo test --no-default-features --features cli,llvm,native --lib fails with E0063 missing field rc_scope_depth in LoopContext at src/llvm/codegen.rs:48066 and :48979 (coverage-only test code). The field was added by d5c9ff6 (R1 refcounting) and the cfg(coverage) tests were not updated, so make coverage / coverage-gate cannot build the lib tests. Fix: add rc_scope_depth: 0 (or the codegen's current depth) to both literals.

Evidence fingerprint: `manual:v1:coverage-cfg-lib-unit-tests-fail-to-compile-loo-bfbcebe3b8a2a693`


## Fix

<unfixed — raised only>

## Notes

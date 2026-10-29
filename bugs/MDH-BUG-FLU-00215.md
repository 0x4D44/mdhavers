# MDH-BUG-FLU-00215 — Native coverage: module_variable_global panics on a missing module ID (regressed in 96d07e4)

- **State:** Open
- **Priority:** Must
- **Severity:** Medium
- **Area:** llvm
- **Raised:** 2026-10-29T00:00:13Z
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
- **State history:** Open (2026-10-29T00:00:13Z, raised via `deltic bugs new --land`)

## Observation

## Observation

Under `--cfg coverage`, two LLVM unit-coverage tests panic with `no entry found for key`:

```bash
RUSTFLAGS="--cfg coverage" CARGO_TARGET_DIR=<wt>/.scratch/coverage-target \
  cargo test --no-default-features --features cli,llvm,native --lib \
  llvm::coverage_tests::llvm_codegen_condition_direct_error_branches_are_exercised_for_unit_coverage
```

Observed: 1 test selected, fails with `no entry found for key` at `src/llvm/codegen.rs` `module_variable_global` (`self.module_unique_ids[import_path]`). A second coverage test reaches the same panic.

Expected: passes, as before 96d07e4.

Cause (from reading the diff): 96d07e4 (fix for 208/210/211) extracted `module_variable_global`, which indexes `module_unique_ids` directly. The code it replaced fell back to `declare_module_init(import_path)`, then to `"module"`, when the ID was missing; the recovery fixture removes the ID deliberately.

Environment: macOS, d36dafb (0.3.117), `--no-default-features --features cli,llvm,native`. Found by the independent Sol verifier of 208-212; cause confirmed by the lead.

Evidence fingerprint: `manual:v1:native-coverage-module-variable-global-panics-o-79e71e65192c14ee`


## Fix

<unfixed — raised only>

## Notes

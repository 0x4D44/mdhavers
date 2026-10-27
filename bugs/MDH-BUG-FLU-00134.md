# MDH-BUG-FLU-00134 — Coverage build (make coverage) fails to compile: 12 errors in coverage-only tests

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** tests
- **Raised:** 2026-10-25T01:59:43Z
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
- **State history:** Open (2026-10-25T01:59:43Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T02:35:19Z, deltic:auto role=fix run=fix-20261025T015951Z-30669772 branch=task/bug-MDH-BUG-FLU-00134-run-fix-20261025T015951Z-30669772 code=8e89bd6 gate=manual) -> Closed (2026-10-27T01:51:48Z, deltic:manual role=verify run=verify-20261026T233955Z-39b8107b verified_fix_run=fix-20261025T015951Z-30669772 verdict=close)

## Observation

Found by lane R0b 2026-10-25, confirmed by the lead: RUSTFLAGS="--cfg coverage" cargo check --no-default-features --features cli,llvm,native --lib --tests (the Makefile coverage configuration) fails with 12 errors. E0509 "cannot move out of type Value, which implements the Drop trait" in coverage-only tests (src/interpreter.rs ~16336, ~24487-24641; tests/tri_instantiation_sweep_coverage_only.rs) since impl Drop for Value landed in 54979e9 (2026-10-23); E0425 dtls_config_from_value / identity_from_pem not found (src/interpreter.rs ~24566, 24639, 24641; these helpers are dtls-gated but the coverage tests calling them are not); E0308 in src/wasm_compiler.rs ~4888; errors in tests/interpreter_dns_stubbed_coverage_only.rs. make coverage / coverage-gate cannot run until fixed.

Evidence fingerprint: `manual:v1:coverage-build-make-coverage-fails-to-compile-1-a025f0ed9ce1b5b8`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:51:48Z, deltic:manual run=verify-20261026T233955Z-39b8107b verified_fix_run=fix-20261025T015951Z-30669772 verdict=close)

Verifier note: Manual verify: LEAD: close - the regression fixture fails on the old LoopJumpInBlockExpr behaviour; the remaining red coverage build was MDH-BUG-FLU-00194 (wasm_tak_drap_strings ungated), fixed in 094641b after this tree's base. Verifier: [lc luna] 8e89bd6: coverage-only interpreter regression passed and old block-jump fixture failed on LoopJumpInBlockExpr, but the recorded coverage check still fails under cli,llvm,native with E0432/E0433 in tests/wasm_tak_drap_strings.rs (requires wasm_runner). Full coverage build remains red.

## Notes

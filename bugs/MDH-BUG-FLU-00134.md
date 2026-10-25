# MDH-BUG-FLU-00134 — Coverage build (make coverage) fails to compile: 12 errors in coverage-only tests

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** tests
- **Raised:** 2026-10-25T01:59:43Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261025T015951Z-30669772
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00134-run-fix-20261025T015951Z-30669772
- **Owner base:** b20d02212f12d4cb657f2c0b63bf405159a24741
- **Owner fingerprint:** -
- **Owner since:** 2026-10-25T01:59:51Z
- **Owner until:** 2026-10-25T05:59:51Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T01:59:43Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane R0b 2026-10-25, confirmed by the lead: RUSTFLAGS="--cfg coverage" cargo check --no-default-features --features cli,llvm,native --lib --tests (the Makefile coverage configuration) fails with 12 errors. E0509 "cannot move out of type Value, which implements the Drop trait" in coverage-only tests (src/interpreter.rs ~16336, ~24487-24641; tests/tri_instantiation_sweep_coverage_only.rs) since impl Drop for Value landed in 54979e9 (2026-10-23); E0425 dtls_config_from_value / identity_from_pem not found (src/interpreter.rs ~24566, 24639, 24641; these helpers are dtls-gated but the coverage tests calling them are not); E0308 in src/wasm_compiler.rs ~4888; errors in tests/interpreter_dns_stubbed_coverage_only.rs. make coverage / coverage-gate cannot run until fixed.

Evidence fingerprint: `manual:v1:coverage-build-make-coverage-fails-to-compile-1-a025f0ed9ce1b5b8`


## Fix

<unfixed — raised only>

## Notes

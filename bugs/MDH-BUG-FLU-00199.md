# MDH-BUG-FLU-00199 — make coverage is red: 4 cfg(coverage) LLVM tests call with wrong arity since 166

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** tests
- **Raised:** 2026-10-27T02:47:37Z
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
- **State history:** Open (2026-10-27T02:47:37Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

On trunk (0.3.98, 2026-10-27 02:45 GMT), `RUSTFLAGS='--cfg coverage' cargo test --lib` (what `make coverage` builds) fails 4 of 1620:

- llvm::codegen::tests::compile_non_native_method_call_static_defaults_for_coverage (src/llvm/codegen.rs:50085):
  TypeError "Function 'm' expects 1 tae 2 arguments but ye gave it 0"
- llvm::codegen::tests::static_method_lookup_cycle_guard_and_best_match_branch_are_exercised_for_coverage (src/llvm/codegen.rs:48164):
  WrongArity { name: "m", expected: 1, got: 0 }
- llvm::coverage_tests::llvm_codegen_more_control_flow_and_object_constructs_are_exercised_for_unit_coverage (src/llvm/coverage_tests.rs:12):
  TypeError "Function 'init' expects 2 tae 3 arguments but ye gave it 1"
- llvm::coverage_tests::llvm_codegen_remaining_instantiation_hotspots_are_exercised_for_unit_coverage (src/llvm/coverage_tests.rs:12):
  WrongArity { name: "muckle", expected: 2, got: 1 }

Cause (inference): MDH-BUG-FLU-00166 (0f7096c, landed c13a562) made wrong arity a native compile error. These cfg(coverage)-only test programs call with too few arguments, so they no longer compile. Plain `cargo test --lib` does not build them, so no lane or lead check saw it. Found while checking a verifier note on 153 (it also saw 4 TLS PermissionDenied failures, which did not reproduce here; likely its sandbox).

Expected: the programs keep exercising the code paths the tests name (static defaults, method lookup, instantiation) with valid arity; `make coverage` is green.

Evidence fingerprint: `manual:v1:make-coverage-is-red-4-cfg-coverage-llvm-tests--bd1763b7b3e048b2`


## Fix

<unfixed — raised only>

## Notes

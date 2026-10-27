# MDH-BUG-FLU-00084 — llvm_compile_repo_modules_coverage overflows the test thread stack

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** tests
- **Raised:** 2026-10-23T03:06:29Z
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
- **State history:** Open (2026-10-23T03:06:29Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:45:31Z, deltic:auto role=fix run=fix-20261023T034526Z-e8262a2d branch=task/bug-MDH-BUG-FLU-00084-run-fix-20261023T034526Z-e8262a2d code=54979e9 gate=manual) -> Closed (2026-10-27T01:51:48Z, deltic:manual role=verify run=verify-20261026T233955Z-8a5020c9 verified_fix_run=fix-20261023T034526Z-e8262a2d verdict=close)

## Observation

Found by the LLVM lane 2026-10-23: the test aborts with a stack overflow on origin/master; it passes with RUST_MIN_STACK=4194304. The test (or the compiler path it drives) should run on a larger thread like the golden harness.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:51:48Z, deltic:manual run=verify-20261026T233955Z-8a5020c9 verified_fix_run=fix-20261023T034526Z-e8262a2d verdict=close)

Verifier note: Manual verify: [lc luna] 54979e9: llvm_compiles_stdlib_and_examples_modules_to_ir_for_coverage passed on 512MiB stack (125.58s); restoring direct call overflowed the stack and aborted with SIGABRT. Mutant restored; source diff clean.

## Notes

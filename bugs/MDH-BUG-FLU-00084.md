# MDH-BUG-FLU-00084 — llvm_compile_repo_modules_coverage overflows the test thread stack

- **State:** Open
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
- **State history:** Open (2026-10-23T03:06:29Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the LLVM lane 2026-10-23: the test aborts with a stack overflow on origin/master; it passes with RUST_MIN_STACK=4194304. The test (or the compiler path it drives) should run on a larger thread like the golden harness.

## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00084 — llvm_compile_repo_modules_coverage overflows the test thread stack

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** tests
- **Raised:** 2026-10-23T03:06:29Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T034526Z-e8262a2d
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00084-run-fix-20261023T034526Z-e8262a2d
- **Owner base:** 54ce8fb0975eae78631a07d2bb0ffcf1eb89f1b4
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:45:26Z
- **Owner until:** 2026-10-23T07:45:26Z
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

# MDH-BUG-FLU-00050 — LLVM integer arithmetic wraps and INT64_MIN / -1 traps

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** llvm
- **Raised:** 2026-10-22T23:24:11Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T164142Z-1e97136d
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00050-run-verify-20261026T164142Z-1e97136d
- **Owner base:** b5d5a9fa852316d9d5ef58b28e50e2449fbe0bad
- **Owner fingerprint:** sha256:7cdbebd9f60f09e235a5c72144a1c17a2a4e41c4568f90cc1331feae15d93a15
- **Owner since:** 2026-10-26T16:41:42Z
- **Owner until:** 2026-10-26T20:41:42Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:04:54Z, deltic:auto role=fix run=fix-20261023T030448Z-77ed95c0 branch=task/bug-MDH-BUG-FLU-00050-run-fix-20261023T030448Z-77ed95c0 code=8e071f4 gate=manual)

## Observation

Inferred: int add/sub/mul wrap silently (interpreter raises IntegerOverflow); only zero is checked for / and %, so MIN/-1 is SIGFPE on x86-64 (codegen compile_binary_int_fast; mdh_runtime.c:344-372). aye + 1 gives 2 where the interpreter raises a type error.

## Fix

<unfixed — raised only>

## Notes

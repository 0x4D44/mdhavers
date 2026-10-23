# MDH-BUG-FLU-00050 — LLVM integer arithmetic wraps and INT64_MIN / -1 traps

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** llvm
- **Raised:** 2026-10-22T23:24:11Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T030448Z-77ed95c0
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00050-run-fix-20261023T030448Z-77ed95c0
- **Owner base:** 3a3734229cf76992b4028b47404b63816b0f64f3
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:04:48Z
- **Owner until:** 2026-10-23T07:04:48Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred: int add/sub/mul wrap silently (interpreter raises IntegerOverflow); only zero is checked for / and %, so MIN/-1 is SIGFPE on x86-64 (codegen compile_binary_int_fast; mdh_runtime.c:344-372). aye + 1 gives 2 where the interpreter raises a type error.

## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00050 — LLVM integer arithmetic wraps and INT64_MIN / -1 traps

- **State:** Closed
- **Priority:** Should
- **Severity:** High
- **Area:** llvm
- **Raised:** 2026-10-22T23:24:11Z
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
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:04:54Z, deltic:auto role=fix run=fix-20261023T030448Z-77ed95c0 branch=task/bug-MDH-BUG-FLU-00050-run-fix-20261023T030448Z-77ed95c0 code=8e071f4 gate=manual) -> Closed (2026-10-26T17:16:10Z, deltic:manual role=verify run=verify-20261026T164142Z-1e97136d verified_fix_run=fix-20261023T030448Z-77ed95c0 verdict=close)

## Observation

Inferred: int add/sub/mul wrap silently (interpreter raises IntegerOverflow); only zero is checked for / and %, so MIN/-1 is SIGFPE on x86-64 (codegen compile_binary_int_fast; mdh_runtime.c:344-372). aye + 1 gives 2 where the interpreter raises a type error.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T17:16:10Z, deltic:manual run=verify-20261026T164142Z-1e97136d verified_fix_run=fix-20261023T030448Z-77ed95c0 verdict=close)

Verifier note: Manual verify: [verifier gpt-6.1-sol (Codex), recorded by lead] 8e071f4: integer_overflow_hurls_instead_of_wrapping passed; CLI caught add/sub/mul/neg; MIN/-1 and bool tests passed. Removing intrinsic overflow hurl failed equality: wrapped -9223372036854775808/-2 instead of caught messages. Restored; diff empty.

## Notes

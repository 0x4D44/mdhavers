# MDH-BUG-FLU-00052 — C runtime string repeat computes len*count unchecked (heap overflow)

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
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:05:18Z, deltic:auto role=fix run=fix-20261023T030512Z-6e409407 branch=task/bug-MDH-BUG-FLU-00052-run-fix-20261023T030512Z-6e409407 code=8e071f4 gate=manual) -> Closed (2026-10-26T17:16:10Z, deltic:manual role=verify run=verify-20261026T164142Z-2a1c4dd0 verified_fix_run=fix-20261023T030512Z-6e409407 verdict=close)

## Observation

Inferred: inline_repeat and mdh_runtime.c:333-337 wrap the size, then strcat-loop overflows; also O(n^2).

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T17:16:10Z, deltic:manual run=verify-20261026T164142Z-2a1c4dd0 verified_fix_run=fix-20261023T030512Z-6e409407 verdict=close)

Verifier note: Manual verify: [verifier gpt-6.1-sol (Codex), recorded by lead] 8e071f4: huge_string_repeat_hurls_instead_of_overflowing passed; CLI/interpreter print repeat caught, ababab, 0. Removing checked repeat-size hurl failed run_ok success assertion: signal termination (None) vs Some(0). Restored; diff empty.

## Notes

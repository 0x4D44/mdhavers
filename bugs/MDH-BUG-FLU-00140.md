# MDH-BUG-FLU-00140 — Native backside_forrit rejects a list that the interpreter reverses

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T06:49:44Z
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
- **State history:** Open (2026-10-25T06:49:44Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T07:13:24Z, deltic:auto role=fix run=fix-20261025T065500Z-77cde6ad branch=task/bug-MDH-BUG-FLU-00140-run-fix-20261025T065500Z-77cde6ad code=bfaa64f gate=manual) -> Closed (2026-10-26T19:40:44Z, deltic:manual role=verify run=verify-20261026T182021Z-2488edcf verified_fix_run=fix-20261025T065500Z-77cde6ad verdict=close)

## Observation

Found by lane N8 2026-10-25 (executed): examples/builtins/prelude_demo.braw exits 1 natively at backside_forrit(numbers) with "Type error in 'reverse_str': got list"; the interpreter (prelude/builtin) accepts lists and exits 0. Native maps backside_forrit to a string-only reverse.

Evidence fingerprint: `manual:v1:native-backside-forrit-rejects-a-list-that-the--bae3094a14b6346d`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T19:40:44Z, deltic:manual run=verify-20261026T182021Z-2488edcf verified_fix_run=fix-20261025T065500Z-77cde6ad verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] fix=bfaa64f; prelude_demo.braw interpreter/native both reversed numbers; backside_forrit_reverses_lists selected and passed 1/1; mutant removing __mdh_reverse_str list branch failed run_ok assertion: Type error in reverse_str: got list, exit Some(1) vs Some(0).

## Notes

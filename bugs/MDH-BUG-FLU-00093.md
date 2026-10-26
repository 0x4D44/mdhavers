# MDH-BUG-FLU-00093 — Native integer overflow error lacks the interpreter's line number

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T04:31:22Z
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
- **State history:** Open (2026-10-23T04:31:22Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T01:43:10Z, deltic:auto role=fix run=fix-20261024T014305Z-ccbcbaf8 branch=task/bug-MDH-BUG-FLU-00093-run-fix-20261024T014305Z-ccbcbaf8 code=5cf1a3f gate=manual) -> Closed (2026-10-26T19:40:44Z, deltic:manual role=verify run=verify-20261026T182021Z-559e66da verified_fix_run=fix-20261024T014305Z-ccbcbaf8 verdict=close)

## Observation

Found by the native hardening lane 2026-10-23: the interpreter says "too muckle at line N - it's overflowed!" (line mid-message); native overflow errors carry no line. Other native runtime type errors ("Och! Type error in 'x'") also still differ from the interpreter's wording.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T19:40:44Z, deltic:manual run=verify-20261026T182021Z-559e66da verified_fix_run=fix-20261024T014305Z-ccbcbaf8 verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] fix=67a0617 (history code 5cf1a3f is version bump); native repro emitted matching overflow lines 3,8; integer_overflow_error_carries_its_line passed 1/1; mutant removing at-line text failed assert_eq! on expected lines 4,9.

## Notes

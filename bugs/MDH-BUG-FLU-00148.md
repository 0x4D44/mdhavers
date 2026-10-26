# MDH-BUG-FLU-00148 — Native rejects wrong-arity builtin calls at compile time; the interpreter raises at run time

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-25T08:13:52Z
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
- **State history:** Open (2026-10-25T08:13:52Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T23:04:25Z, deltic:auto role=fix run=fix-20261025T184736Z-eeee3595 branch=task/bug-MDH-BUG-FLU-00148-run-fix-20261025T184736Z-eeee3595 code=542e76b gate=manual) -> Closed (2026-10-26T19:40:44Z, deltic:manual role=verify run=verify-20261026T182021Z-dbe8e5d3 verified_fix_run=fix-20261025T184736Z-eeee3595 verdict=close)

## Observation

Found by lane S10's builtin audit 2026-10-25 (executed): wrong-arity calls to gaun, sieve, hunt, ilk, aw, grup_up, pairt_by, tumble, center, pad_left, pad_right, log_event and log_init are native compile errors, while the interpreter raises a catchable runtime error. Same question as MDH-BUG-FLU-00138 (compile-time vs run-time errors); decide both together.

Evidence fingerprint: `manual:v1:native-rejects-wrong-arity-builtin-calls-at-com-0334a949873c1e2a`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T19:40:44Z, deltic:manual run=verify-20261026T182021Z-dbe8e5d3 verified_fix_run=fix-20261025T184736Z-eeee3595 verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] fix=542e76b (compile-time arity is accepted policy); gaun([]) is caught by interpreter at runtime and rejected natively as expects 2 arguments; llvm_call_arity selected 5/5; CLI docs and #SKIP_NATIVE align, pre-fix docs lacked policy. Docs-only: no hunk or dedicated regression mutant.

## Notes

# MDH-BUG-FLU-00164 — Native: unbounded recursion segfaults instead of raising the stack error

- **State:** Closed
- **Priority:** Should
- **Severity:** High
- **Area:** native
- **Raised:** 2026-10-26T05:03:22Z
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
- **State history:** Open (2026-10-26T05:03:22Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T10:35:53Z, deltic:auto role=fix run=fix-20261026T050440Z-fef125a9 branch=task/bug-MDH-BUG-FLU-00164-run-fix-20261026T050440Z-fef125a9 code=cc7bfde gate=manual) -> Closed (2026-10-26T17:16:10Z, deltic:manual role=verify run=verify-20261026T164142Z-6040ede5 verified_fix_run=fix-20261026T050440Z-fef125a9 verdict=close)

## Observation

Found by lane H1 2026-10-26, reproduced by the lead: `dae r(n) { gie 1 + r(n + 1) }` / `blether r(0)` exits 1 with "Stack's fair puggled! Too many nested calls ... yer recursion's gone radge!" in the interpreter; natively (also MDH_RC=1) it dies with SIGSEGV, exit 139. A finite depth of 100000 does the same. Fix: a depth counter or stack-pointer check in function prologues (or a guard page handler) that raises the interpreter's Scots error as a catchable runtime error; test both unbounded and deep-but-finite recursion; measure the prologue cost on fib.

Evidence fingerprint: `manual:v1:native-unbounded-recursion-segfaults-instead-of-42981c3ed16f5ff9`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T17:16:10Z, deltic:manual run=verify-20261026T164142Z-6040ede5 verified_fix_run=fix-20261026T050440Z-fef125a9 verdict=close)

Verifier note: Manual verify: [verifier gpt-6.1-sol (Codex), recorded by lead] cc7bfde (guard 1d23ed3): unbounded_recursion_raises_the_scots_error_and_exits_1 passed plain/RC; catch/retry/thread/signal tests passed. CLI unbounded and depth 100000 raise Scots stack error, exit 1. Disabling guard init failed exit assertion None vs Some(1). Restored; diff empty.

## Notes

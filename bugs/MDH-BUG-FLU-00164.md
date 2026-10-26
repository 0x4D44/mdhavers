# MDH-BUG-FLU-00164 — Native: unbounded recursion segfaults instead of raising the stack error

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** native
- **Raised:** 2026-10-26T05:03:22Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261026T050440Z-fef125a9
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00164-run-fix-20261026T050440Z-fef125a9
- **Owner base:** 0ce2a5bc5cf9ba91c13f5b84148dcd8cfea0ba99
- **Owner fingerprint:** -
- **Owner since:** 2026-10-26T05:04:40Z
- **Owner until:** 2026-10-26T09:04:40Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T05:03:22Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane H1 2026-10-26, reproduced by the lead: `dae r(n) { gie 1 + r(n + 1) }` / `blether r(0)` exits 1 with "Stack's fair puggled! Too many nested calls ... yer recursion's gone radge!" in the interpreter; natively (also MDH_RC=1) it dies with SIGSEGV, exit 139. A finite depth of 100000 does the same. Fix: a depth counter or stack-pointer check in function prologues (or a guard page handler) that raises the interpreter's Scots error as a catchable runtime error; test both unbounded and deep-but-finite recursion; measure the prologue cost on fib.

Evidence fingerprint: `manual:v1:native-unbounded-recursion-segfaults-instead-of-42981c3ed16f5ff9`


## Fix

<unfixed — raised only>

## Notes

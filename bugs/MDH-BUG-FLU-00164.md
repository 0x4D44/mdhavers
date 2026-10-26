# MDH-BUG-FLU-00164 — Native: unbounded recursion segfaults instead of raising the stack error

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** native
- **Raised:** 2026-10-26T05:03:22Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T164142Z-6040ede5
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00164-run-verify-20261026T164142Z-6040ede5
- **Owner base:** b5d5a9fa852316d9d5ef58b28e50e2449fbe0bad
- **Owner fingerprint:** sha256:3ab3f917e50495eace0facf651e90f6e372f8d9eb040e89e24cbea9300685906
- **Owner since:** 2026-10-26T16:41:42Z
- **Owner until:** 2026-10-26T20:41:42Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T05:03:22Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T10:35:53Z, deltic:auto role=fix run=fix-20261026T050440Z-fef125a9 branch=task/bug-MDH-BUG-FLU-00164-run-fix-20261026T050440Z-fef125a9 code=cc7bfde gate=manual)

## Observation

Found by lane H1 2026-10-26, reproduced by the lead: `dae r(n) { gie 1 + r(n + 1) }` / `blether r(0)` exits 1 with "Stack's fair puggled! Too many nested calls ... yer recursion's gone radge!" in the interpreter; natively (also MDH_RC=1) it dies with SIGSEGV, exit 139. A finite depth of 100000 does the same. Fix: a depth counter or stack-pointer check in function prologues (or a guard page handler) that raises the interpreter's Scots error as a catchable runtime error; test both unbounded and deep-but-finite recursion; measure the prologue cost on fib.

Evidence fingerprint: `manual:v1:native-unbounded-recursion-segfaults-instead-of-42981c3ed16f5ff9`


## Fix

<unfixed — raised only>

## Notes

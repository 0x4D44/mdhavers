# MDH-BUG-FLU-00001 — Deep recursion aborts the process with a native stack overflow (no StackOverflow error)

- **State:** Closed
- **Priority:** Should
- **Severity:** High
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:06Z
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
- **State history:** Open (2026-10-22T23:24:06Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:48:01Z, deltic:auto role=fix run=fix-20261023T014755Z-43fd29fa branch=task/bug-MDH-BUG-FLU-00001-run-fix-20261023T014755Z-43fd29fa code=d84823a gate=manual) -> Closed (2026-10-26T22:19:14Z, deltic:manual role=verify run=verify-20261026T214519Z-86db763a verified_fix_run=fix-20261023T014755Z-43fd29fa verdict=close)

## Observation

Confirmed 2026-10-23 on macOS debug build: a recursive dae with depth in the thousands, and infinite recursion, print "thread 'main' has overflowed its stack / fatal runtime error" and exit 134. No depth counter exists anywhere in src/interpreter.rs; HaversError::StackOverflow is never constructed, and hae_a_bash cannot catch it. Repro: .scratch repro interp/deep-recursion.braw (recursive function with no base case).

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T22:19:14Z, deltic:manual run=verify-20261026T214519Z-86db763a verified_fix_run=fix-20261023T014755Z-43fd29fa verdict=close)

Verifier note: Manual verify: [verifier gpt-6.1-sol (Codex), recorded by lead] d84823a; cli_run_unbounded_recursion_gives_scots_stack_error selected/pass; guard disabled -> assertion exit -1 != 1, stderr native stack overflow. Original infinite-recursion CLI repro catches Stack's fair puggled and prints still alive; source restored, diff empty.

## Notes

# MDH-BUG-FLU-00001 — Deep recursion aborts the process with a native stack overflow (no StackOverflow error)

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:06Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-86db763a
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00001-run-verify-20261026T214519Z-86db763a
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:d19f8c3516b598e3822dbfe40dd84047617357f6c9439d7a5806c2d4c8aeda92
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:06Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:48:01Z, deltic:auto role=fix run=fix-20261023T014755Z-43fd29fa branch=task/bug-MDH-BUG-FLU-00001-run-fix-20261023T014755Z-43fd29fa code=d84823a gate=manual)

## Observation

Confirmed 2026-10-23 on macOS debug build: a recursive dae with depth in the thousands, and infinite recursion, print "thread 'main' has overflowed its stack / fatal runtime error" and exit 134. No depth counter exists anywhere in src/interpreter.rs; HaversError::StackOverflow is never constructed, and hae_a_bash cannot catch it. Repro: .scratch repro interp/deep-recursion.braw (recursive function with no base case).

## Fix

<unfixed — raised only>

## Notes

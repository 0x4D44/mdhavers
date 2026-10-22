# MDH-BUG-FLU-00001 — Deep recursion aborts the process with a native stack overflow (no StackOverflow error)

- **State:** Open
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
- **State history:** Open (2026-10-22T23:24:06Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed 2026-10-23 on macOS debug build: a recursive dae with depth in the thousands, and infinite recursion, print "thread 'main' has overflowed its stack / fatal runtime error" and exit 134. No depth counter exists anywhere in src/interpreter.rs; HaversError::StackOverflow is never constructed, and hae_a_bash cannot catch it. Repro: .scratch repro interp/deep-recursion.braw (recursive function with no base case).

## Fix

<unfixed — raised only>

## Notes

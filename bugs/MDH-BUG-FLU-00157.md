# MDH-BUG-FLU-00157 — Native build of examples/stdlib/game_demo.braw times out after 120 seconds

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T23:04:55Z
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
- **State history:** Open (2026-10-25T23:04:55Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane S15 2026-10-25 during a native build sweep of examples/ (observed once under load ~90, not rerun): mdhavers build examples/stdlib/game_demo.braw did not finish within 120 s, while other examples build in seconds. Possible compile-time blow-up (prelude/stdlib fixpoint injection, a huge function, or an optimiser pass). Reproduce with a timing at -O0 and -O2 and --emit-llvm size; fix the cause.

Evidence fingerprint: `manual:v1:native-build-of-examples-stdlib-game-demo-braw--5dc229ae0518b37d`


## Fix

<unfixed — raised only>

## Notes

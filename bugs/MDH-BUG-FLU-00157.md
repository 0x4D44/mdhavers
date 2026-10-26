# MDH-BUG-FLU-00157 — Native build of examples/stdlib/game_demo.braw times out after 120 seconds

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T23:04:55Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T194306Z-f95e041b
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00157-run-verify-20261026T194306Z-f95e041b
- **Owner base:** 36750bc8afd262a99873992d919d9ed250946aef
- **Owner fingerprint:** sha256:f19dbca6bb2248a4c18c8bc065c035a9e8ef13f0e580130c38ae1d34a683fccd
- **Owner since:** 2026-10-26T19:43:06Z
- **Owner until:** 2026-10-26T23:43:06Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T23:04:55Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T01:31:35Z, deltic:auto role=fix run=fix-20261025T230506Z-6b2db5bc branch=task/bug-MDH-BUG-FLU-00157-run-fix-20261025T230506Z-6b2db5bc code=8b0ec87 gate=manual)

## Observation

Found by lane S15 2026-10-25 during a native build sweep of examples/ (observed once under load ~90, not rerun): mdhavers build examples/stdlib/game_demo.braw did not finish within 120 s, while other examples build in seconds. Possible compile-time blow-up (prelude/stdlib fixpoint injection, a huge function, or an optimiser pass). Reproduce with a timing at -O0 and -O2 and --emit-llvm size; fix the cause.

Evidence fingerprint: `manual:v1:native-build-of-examples-stdlib-game-demo-braw--5dc229ae0518b37d`


## Fix

<unfixed — raised only>

## Notes

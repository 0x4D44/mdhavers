# MDH-BUG-FLU-00191 — stdlib input helpers exit(0) at end of input instead of returning naething

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** stdlib
- **Raised:** 2026-10-26T15:32:33Z
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
- **State history:** Open (2026-10-26T15:32:33Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T22:26:23Z, deltic:auto role=fix run=fix-20261026T222536Z-ea4f2380 branch=task/bug-MDH-BUG-FLU-00191-run-fix-20261026T222536Z-ea4f2380 code=cc02a5f gate=manual) -> Closed (2026-10-27T02:35:24Z, deltic:manual role=verify run=verify-20261027T013251Z-96b91177 verified_fix_run=fix-20261026T222536Z-ea4f2380 verdict=close)

## Observation

Arthur decision round 4 (2026-10-26): stdlib input helpers that read stdin (e.g. the adventure_speir_or_awa pattern; find all in stdlib/ and examples/ helpers) call exit(0) at end of input, so programs quit silently mid-flow. Make them return naething at EOF, consistent with speir (decision 136); update callers/examples so they end cleanly; tests on interpreter and native.

Evidence fingerprint: `manual:v1:stdlib-input-helpers-exit-0-at-end-of-input-ins-1d1c7f2da9b0b44f`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T02:35:24Z, deltic:manual run=verify-20261027T013251Z-96b91177 verified_fix_run=fix-20261026T222536Z-ea4f2380 verdict=close)

Verifier note: Manual verify: [le luna] fix cc02a5f; eof_helpers passed 4/4 interpreter tests; restoring exit(0) in cli_speir_or_awa made cli_helpers_return_naething_at_end_of_input_in_the_interpreter fail its expected answers assertion (no “still here”).

## Notes

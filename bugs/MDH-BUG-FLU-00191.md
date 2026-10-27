# MDH-BUG-FLU-00191 — stdlib input helpers exit(0) at end of input instead of returning naething

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** stdlib
- **Raised:** 2026-10-26T15:32:33Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261027T013251Z-96b91177
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00191-run-verify-20261027T013251Z-96b91177
- **Owner base:** dc207b08e001e4c28ffbe641571b9bde4a9b16c3
- **Owner fingerprint:** sha256:ac09dec5d5302c0add3adbf412664757e0461b1889c82adc3d3eb1acc95699a8
- **Owner since:** 2026-10-27T01:32:51Z
- **Owner until:** 2026-10-27T05:32:51Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T15:32:33Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T22:26:23Z, deltic:auto role=fix run=fix-20261026T222536Z-ea4f2380 branch=task/bug-MDH-BUG-FLU-00191-run-fix-20261026T222536Z-ea4f2380 code=cc02a5f gate=manual)

## Observation

Arthur decision round 4 (2026-10-26): stdlib input helpers that read stdin (e.g. the adventure_speir_or_awa pattern; find all in stdlib/ and examples/ helpers) call exit(0) at end of input, so programs quit silently mid-flow. Make them return naething at EOF, consistent with speir (decision 136); update callers/examples so they end cleanly; tests on interpreter and native.

Evidence fingerprint: `manual:v1:stdlib-input-helpers-exit-0-at-end-of-input-ins-1d1c7f2da9b0b44f`


## Fix

<unfixed — raised only>

## Notes

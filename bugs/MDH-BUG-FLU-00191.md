# MDH-BUG-FLU-00191 — stdlib input helpers exit(0) at end of input instead of returning naething

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** stdlib
- **Raised:** 2026-10-26T15:32:33Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261026T222536Z-ea4f2380
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00191-run-fix-20261026T222536Z-ea4f2380
- **Owner base:** e048d5c83235a33282b10f9f210d567729ec19b4
- **Owner fingerprint:** -
- **Owner since:** 2026-10-26T22:25:36Z
- **Owner until:** 2026-10-27T02:25:36Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T15:32:33Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Arthur decision round 4 (2026-10-26): stdlib input helpers that read stdin (e.g. the adventure_speir_or_awa pattern; find all in stdlib/ and examples/ helpers) call exit(0) at end of input, so programs quit silently mid-flow. Make them return naething at EOF, consistent with speir (decision 136); update callers/examples so they end cleanly; tests on interpreter and native.

Evidence fingerprint: `manual:v1:stdlib-input-helpers-exit-0-at-end-of-input-ins-1d1c7f2da9b0b44f`


## Fix

<unfixed — raised only>

## Notes

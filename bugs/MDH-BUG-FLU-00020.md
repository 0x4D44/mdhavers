# MDH-BUG-FLU-00020 — Integer literal overflow reported as unknown token

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-22T23:24:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T233955Z-6f3a6fec
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00020-run-verify-20261026T233955Z-6f3a6fec
- **Owner base:** 3a7321efd6af1e16d34dcd7fed8e7df5ad38c691
- **Owner fingerprint:** sha256:2f53ce774a1a81340296dd4a1031ffa2a9629472cea7478a96601e924b568d2a
- **Owner since:** 2026-10-26T23:39:55Z
- **Owner until:** 2026-10-27T03:39:55Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:51:19Z, deltic:auto role=fix run=fix-20261023T015114Z-220e94ff branch=task/bug-MDH-BUG-FLU-00020-run-fix-20261023T015114Z-220e94ff code=8204122 gate=manual)

## Observation

Confirmed: `ken big = 9223372036854775808` -> "Ah dinnae ken whit '9223372036854775808' is". token.rs:159 callback returns None; InvalidNumber exists.

## Fix

<unfixed — raised only>

## Notes

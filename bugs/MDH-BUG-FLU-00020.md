# MDH-BUG-FLU-00020 — Integer literal overflow reported as unknown token

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-22T23:24:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T015114Z-220e94ff
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00020-run-fix-20261023T015114Z-220e94ff
- **Owner base:** 33526be707aec162ae19ba6e92e40deaa882b981
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:51:14Z
- **Owner until:** 2026-10-23T05:51:14Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: `ken big = 9223372036854775808` -> "Ah dinnae ken whit '9223372036854775808' is". token.rs:159 callback returns None; InvalidNumber exists.

## Fix

<unfixed — raised only>

## Notes

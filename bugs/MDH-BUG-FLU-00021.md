# MDH-BUG-FLU-00021 — F-string holes silently drop trailing tokens and report errors at line 1

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** parser
- **Raised:** 2026-10-22T23:24:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T015124Z-d729e4e3
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00021-run-fix-20261023T015124Z-d729e4e3
- **Owner base:** 06e6a4d894be0f7aa0434a9683df12babcabdb48
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:51:24Z
- **Owner until:** 2026-10-23T05:51:24Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: f"{a b}" prints a; f"{x:>5}" ignores the spec (parser.rs:1591 never checks the sub-parser reached EOF). An error inside a hole on line 4 is reported at line 1 (re-lexed from scratch, parser.rs:1589).

## Fix

<unfixed — raised only>

## Notes

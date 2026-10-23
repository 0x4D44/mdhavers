# MDH-BUG-FLU-00029 — mdhavers fmt does not re-escape string literals

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** formatter
- **Raised:** 2026-10-22T23:24:09Z
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
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:58:22Z, deltic:auto role=fix run=fix-20261023T015816Z-fa4529d4 branch=task/bug-MDH-BUG-FLU-00029-run-fix-20261023T015816Z-fa4529d4 code=6e06c31 gate=manual)

## Observation

Confirmed: "say \"hi\"" is written as "say "hi"" (parse error); "back\\nslash" becomes a real newline (meaning change); single-quoted strings containing " break. ast.rs:411 Literal Display.

## Fix

<unfixed — raised only>

## Notes

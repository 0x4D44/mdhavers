# MDH-BUG-FLU-00029 — mdhavers fmt does not re-escape string literals

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** formatter
- **Raised:** 2026-10-22T23:24:09Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-184b001a
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00029-run-verify-20261026T214519Z-184b001a
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:90108a6884e82f3a02a1502cf4ffa223a7ea3da744a4108b59da32e9bdb118f0
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
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

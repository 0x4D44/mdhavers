# MDH-BUG-FLU-00031 — mdhavers fmt breaks f-strings ({{ }} escapes and inner quotes)

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** formatter
- **Raised:** 2026-10-22T23:24:09Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-fe347781
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00031-run-verify-20261026T214519Z-fe347781
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:a63b32adf2251b25d11147a7d85f5c7b0b7f7d57327b0ac638dbe5da4725b61e
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:58:43Z, deltic:auto role=fix run=fix-20261023T015838Z-1de6b86d branch=task/bug-MDH-BUG-FLU-00031-run-fix-20261023T015838Z-1de6b86d code=6e06c31 gate=manual)

## Observation

Confirmed: f"{{braces}} {d['k']}" -> f"{braces} {d["k"]}" which no longer parses (formatter.rs:567).

## Fix

<unfixed — raised only>

## Notes

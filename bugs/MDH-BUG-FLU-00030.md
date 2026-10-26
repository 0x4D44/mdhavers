# MDH-BUG-FLU-00030 — mdhavers fmt turns float literals into integers

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** formatter
- **Raised:** 2026-10-22T23:24:09Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-2853a67f
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00030-run-verify-20261026T214519Z-2853a67f
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:bc772597211e23d85fa1167b6ff4e018868791126b37caf9a6964823f00ed58f
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:58:32Z, deltic:auto role=fix run=fix-20261023T015827Z-85bf68c7 branch=task/bug-MDH-BUG-FLU-00030-run-fix-20261023T015827Z-85bf68c7 code=6e06c31 gate=manual)

## Observation

Confirmed: 1.0 is written as 1 (changes / semantics), 1e300 as a 301-digit integer that no longer lexes. ast.rs:410 f64 Display.

## Fix

<unfixed — raised only>

## Notes

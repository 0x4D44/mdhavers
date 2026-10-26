# MDH-BUG-FLU-00030 — mdhavers fmt turns float literals into integers

- **State:** Closed
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
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:58:32Z, deltic:auto role=fix run=fix-20261023T015827Z-85bf68c7 branch=task/bug-MDH-BUG-FLU-00030-run-fix-20261023T015827Z-85bf68c7 code=6e06c31 gate=manual) -> Closed (2026-10-26T22:40:14Z, deltic:manual role=verify run=verify-20261026T214519Z-2853a67f verified_fix_run=fix-20261023T015827Z-85bf68c7 verdict=close)

## Observation

Confirmed: 1.0 is written as 1 (changes / semantics), 1e300 as a 301-digit integer that no longer lexes. ast.rs:410 f64 Display.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T22:40:14Z, deltic:manual run=verify-20261026T214519Z-2853a67f verified_fix_run=fix-20261023T015827Z-85bf68c7 verdict=close)

Verifier note: Manual verify: [o1b sol] 6e06c31: fmt_floats_stay_floats selected/pass; float Debug reverted to Display; checked-format assertion refuses huge integer literal. CLI keeps 1.0 and 1e300 and --check passes; restored clean.

## Notes

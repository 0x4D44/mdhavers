# MDH-BUG-FLU-00006 — Inclusive range to i64::MAX and huge slice steps overflow (panic)

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-067203a4
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00006-run-verify-20261026T214519Z-067203a4
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:98f6d04b64813b0714d18b7e5046147e7d6b6e80a11db47f29d4ad48a5f0c760
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:48:58Z, deltic:auto role=fix run=fix-20261023T014852Z-7ffbfd38 branch=task/bug-MDH-BUG-FLU-00006-run-fix-20261023T014852Z-7ffbfd38 code=d84823a gate=manual)

## Observation

Confirmed: `0..=9223372036854775807` panics "attempt to add with overflow" at interpreter.rs ~11224; xs[1:3:i64::MAX] panics at ~11683. Ranges are also materialised eagerly so `fer i in 0..10000000000 { brak }` exhausts memory (not run, memory hog).

## Fix

<unfixed — raised only>

## Notes

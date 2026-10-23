# MDH-BUG-FLU-00006 — Inclusive range to i64::MAX and huge slice steps overflow (panic)

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T014852Z-7ffbfd38
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00006-run-fix-20261023T014852Z-7ffbfd38
- **Owner base:** 09a3c5dc32d9e12dd46d449fe73907d11ba19654
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:48:52Z
- **Owner until:** 2026-10-23T05:48:52Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: `0..=9223372036854775807` panics "attempt to add with overflow" at interpreter.rs ~11224; xs[1:3:i64::MAX] panics at ~11683. Ranges are also materialised eagerly so `fer i in 0..10000000000 { brak }` exhausts memory (not run, memory hog).

## Fix

<unfixed — raised only>

## Notes

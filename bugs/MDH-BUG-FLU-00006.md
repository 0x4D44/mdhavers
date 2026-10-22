# MDH-BUG-FLU-00006 — Inclusive range to i64::MAX and huge slice steps overflow (panic)

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:07Z
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
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: `0..=9223372036854775807` panics "attempt to add with overflow" at interpreter.rs ~11224; xs[1:3:i64::MAX] panics at ~11683. Ranges are also materialised eagerly so `fer i in 0..10000000000 { brak }` exhausts memory (not run, memory hog).

## Fix

<unfixed — raised only>

## Notes

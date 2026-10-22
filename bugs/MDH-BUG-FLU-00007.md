# MDH-BUG-FLU-00007 — Operator-overload method that mutates masel panics with RefCell already borrowed

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

Confirmed: a kin with __pit_thegither__ that assigns masel.x panics "RefCell already borrowed" at interpreter.rs ~11462: `if let Some(m) = inst.borrow().class.find_method(..)` keeps the Ref alive through the body (edition 2021 temporaries).

## Fix

<unfixed — raised only>

## Notes

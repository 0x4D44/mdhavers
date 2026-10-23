# MDH-BUG-FLU-00007 — Operator-overload method that mutates masel panics with RefCell already borrowed

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T014903Z-5c49cbe2
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00007-run-fix-20261023T014903Z-5c49cbe2
- **Owner base:** b190dc28f5ee1e14653b266b0e1a98119067bd8a
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:49:03Z
- **Owner until:** 2026-10-23T05:49:03Z
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

# MDH-BUG-FLU-00007 — Operator-overload method that mutates masel panics with RefCell already borrowed

- **State:** Closed
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
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:49:09Z, deltic:auto role=fix run=fix-20261023T014903Z-5c49cbe2 branch=task/bug-MDH-BUG-FLU-00007-run-fix-20261023T014903Z-5c49cbe2 code=d84823a gate=manual) -> Closed (2026-10-26T22:19:14Z, deltic:manual role=verify run=verify-20261026T214519Z-9903e0c1 verified_fix_run=fix-20261023T014903Z-5c49cbe2 verdict=close)

## Observation

Confirmed: a kin with __pit_thegither__ that assigns masel.x panics "RefCell already borrowed" at interpreter.rs ~11462: `if let Some(m) = inst.borrow().class.find_method(..)` keeps the Ref alive through the body (edition 2021 temporaries).

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T22:19:14Z, deltic:manual run=verify-20261026T214519Z-9903e0c1 verified_fix_run=fix-20261023T014903Z-5c49cbe2 verdict=close)

Verifier note: Manual verify: [verifier gpt-6.1-sol (Codex), recorded by lead] d84823a; operator_overload_that_mutates_masel_works selected/pass; restore borrow across overload call -> RefCell already borrowed at masel assignment; scratch overload CLI assertion 101 != 0. Original __pit_thegither__ mutates masel and prints 1. Restored; diff empty.

## Notes

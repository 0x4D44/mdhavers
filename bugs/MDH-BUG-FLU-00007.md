# MDH-BUG-FLU-00007 — Operator-overload method that mutates masel panics with RefCell already borrowed

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-9903e0c1
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00007-run-verify-20261026T214519Z-9903e0c1
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:19094612cd52b85df49c7f7170c4ed9cc7ade8f6add70a81f957f5e0a3661bd0
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:49:09Z, deltic:auto role=fix run=fix-20261023T014903Z-5c49cbe2 branch=task/bug-MDH-BUG-FLU-00007-run-fix-20261023T014903Z-5c49cbe2 code=d84823a gate=manual)

## Observation

Confirmed: a kin with __pit_thegither__ that assigns masel.x panics "RefCell already borrowed" at interpreter.rs ~11462: `if let Some(m) = inst.borrow().class.find_method(..)` keeps the Ref alive through the body (edition 2021 temporaries).

## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00008 — bytes_append(b, b) panics with RefCell already borrowed

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-00a73f0c
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00008-run-verify-20261026T214519Z-00a73f0c
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:9f0703e85cfd23c17fb1352bb8d60dec5a0a51239afef452419a4a03db0ed3ca
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:49:19Z, deltic:auto role=fix run=fix-20261023T014914Z-c57ec52b branch=task/bug-MDH-BUG-FLU-00008-run-fix-20261023T014914Z-c57ec52b code=d84823a gate=manual)

## Observation

Confirmed: panics at interpreter.rs ~2760; other.borrow() held while borrow_mut() on the same Rc.

## Fix

<unfixed — raised only>

## Notes

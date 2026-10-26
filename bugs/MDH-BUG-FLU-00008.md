# MDH-BUG-FLU-00008 — bytes_append(b, b) panics with RefCell already borrowed

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
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:49:19Z, deltic:auto role=fix run=fix-20261023T014914Z-c57ec52b branch=task/bug-MDH-BUG-FLU-00008-run-fix-20261023T014914Z-c57ec52b code=d84823a gate=manual) -> Closed (2026-10-26T22:19:14Z, deltic:manual role=verify run=verify-20261026T214519Z-00a73f0c verified_fix_run=fix-20261023T014914Z-c57ec52b verdict=close)

## Observation

Confirmed: panics at interpreter.rs ~2760; other.borrow() held while borrow_mut() on the same Rc.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T22:19:14Z, deltic:manual run=verify-20261026T214519Z-00a73f0c verified_fix_run=fix-20261023T014914Z-c57ec52b verdict=close)

Verifier note: Manual verify: [verifier gpt-6.1-sol (Codex), recorded by lead] d84823a; bytes_append_to_itself_doubles_the_buffer selected/pass; revert input snapshot to held Ref -> RefCell already borrowed; scratch CLI bytes self-append assertion 101 != 0. Original bytes_append(b,b) prints length 4 and copied byte 7. Restored; diff empty.

## Notes

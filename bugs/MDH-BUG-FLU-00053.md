# MDH-BUG-FLU-00053 — C runtime dict insert copies the whole dict and never frees (quadratic)

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** llvm
- **Raised:** 2026-10-22T23:24:11Z
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
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T04:30:47Z, deltic:auto role=fix run=fix-20261023T043041Z-aa3781ce branch=task/bug-MDH-BUG-FLU-00053-run-fix-20261023T043041Z-aa3781ce code=c9f3fd5 gate=manual) -> Closed (2026-10-26T18:20:05Z, deltic:manual role=verify run=verify-20261026T171637Z-785d6727 verified_fix_run=fix-20261023T043041Z-aa3781ce verdict=close)

## Observation

Inferred: mdh_runtime.c:3857-3875 with gc_stub (no free).

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T18:20:05Z, deltic:manual run=verify-20261026T171637Z-785d6727 verified_fix_run=fix-20261023T043041Z-aa3781ce verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] c9f3fd5; 53.braw -> 200000/199999/200000; two_hundred_thousand_dict_inserts_finish_quickly: 1 passed; mutant forced per-insert realloc -> assertion expected Some(0), got None after 10.21s.

## Notes

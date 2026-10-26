# MDH-BUG-FLU-00184 — Native runtime: string buffer ignores allocation failure and crashes on out-of-memory

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-26T08:44:39Z
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
- **State history:** Open (2026-10-26T08:44:39Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T12:20:03Z, deltic:auto role=fix run=fix-20261026T103609Z-be992a44 branch=task/bug-MDH-BUG-FLU-00184-run-fix-20261026T103609Z-be992a44 code=26cd19d gate=manual) -> Closed (2026-10-26T21:44:08Z, deltic:manual role=verify run=verify-20261026T194306Z-374fbc16 verified_fix_run=fix-20261026T103609Z-be992a44 verdict=close)

## Observation

Found in Astra review of MDH-BUG-FLU-00164 (source analysis, 2026-10-26, not executed): the C string-buffer helpers in runtime/mdh_runtime.c (~:200, :207) use allocation helpers from runtime/gc_stub.c (~:161, :236) that return NULL on failure, and do not check it, so out-of-memory while formatting writes through NULL and crashes instead of raising a catchable error. It also means __mdh_to_string_checked's quiet-mode restore is not exception-safe in principle. Fix: check allocations in the string buffer and fail through the normal error path (outside quiet mode) or report status (inside it).

Evidence fingerprint: `manual:v1:native-runtime-string-buffer-ignores-allocation-a51132498b5ac488`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T21:44:08Z, deltic:manual run=verify-20261026T194306Z-374fbc16 verified_fix_run=fix-20261026T103609Z-be992a44 verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] 26cd19d; both running_out_of_memory tests selected/pass. Disabling the non-quiet formatter raise made running_out_of_memory_formatting_raises_the_too_big_error fail output assertion: 440311 instead of the too-big error.

## Notes

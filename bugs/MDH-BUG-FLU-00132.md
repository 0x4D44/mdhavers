# MDH-BUG-FLU-00132 — Native chunks(): signed overflow computing the chunk count and loop step for a huge size

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-25T01:32:27Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261026T214549Z-db933061
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00132-run-fix-20261026T214549Z-db933061
- **Owner base:** e9aacdb5b2f66f7b00214d225728b201140c07cf
- **Owner fingerprint:** -
- **Owner since:** 2026-10-26T21:45:49Z
- **Owner until:** 2026-10-27T01:45:49Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=1, indeterminate=0
- **State history:** Open (2026-10-25T01:32:27Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T02:16:04Z, deltic:auto role=fix run=fix-20261025T020015Z-66c23764 branch=task/bug-MDH-BUG-FLU-00132-run-fix-20261025T020015Z-66c23764 code=6a72241 gate=manual) -> Open (2026-10-26T19:40:44Z, deltic:manual role=verify run=verify-20261026T182021Z-3524a0ae verified_fix_run=fix-20261025T020015Z-66c23764 verdict=doubt reason=reopen-verifier-gpt-6-luna-max-codex-recorded-by-lead-fix6a72241-interpreter-nat)

## Observation

Found by lane R0a 2026-10-25 (read), checked by the lead: __mdh_chunks (runtime/mdh_runtime.c ~9051) computes (len + n - 1) / n and steps i += n / end = i + n in int64; with n near INT64_MAX these overflow (undefined behaviour in C). Repro sketch: chunks([1, 2, 3], 9223372036854775807). The interpreter returns one chunk. Fix: compute the count as len / n + (len % n != 0) and clamp end with a subtraction check.

Evidence fingerprint: `manual:v1:native-chunks-signed-overflow-computing-the-chu-343de05d68adc9e5`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T19:40:44Z, deltic:manual run=verify-20261026T182021Z-3524a0ae verified_fix_run=fix-20261025T020015Z-66c23764 verdict=doubt)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] fix=6a72241; interpreter/native repro printed [[1, 2, 3]]; chunks_with_a_huge_size_does_not_overflow passed 1/1 both before and after reverting to (len+n-1)/n arithmetic; test does not cover the fix.

## Notes

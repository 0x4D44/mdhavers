# MDH-BUG-FLU-00132 — Native chunks(): signed overflow computing the chunk count and loop step for a huge size

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-25T01:32:27Z
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
- **State history:** Open (2026-10-25T01:32:27Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T02:16:04Z, deltic:auto role=fix run=fix-20261025T020015Z-66c23764 branch=task/bug-MDH-BUG-FLU-00132-run-fix-20261025T020015Z-66c23764 code=6a72241 gate=manual)

## Observation

Found by lane R0a 2026-10-25 (read), checked by the lead: __mdh_chunks (runtime/mdh_runtime.c ~9051) computes (len + n - 1) / n and steps i += n / end = i + n in int64; with n near INT64_MAX these overflow (undefined behaviour in C). Repro sketch: chunks([1, 2, 3], 9223372036854775807). The interpreter returns one chunk. Fix: compute the count as len / n + (len % n != 0) and clamp end with a subtraction check.

Evidence fingerprint: `manual:v1:native-chunks-signed-overflow-computing-the-chu-343de05d68adc9e5`


## Fix

<unfixed — raised only>

## Notes

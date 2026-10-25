# MDH-BUG-FLU-00132 — Native chunks(): signed overflow computing the chunk count and loop step for a huge size

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-25T01:32:27Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261025T020015Z-66c23764
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00132-run-fix-20261025T020015Z-66c23764
- **Owner base:** eb244602852300eaa2d7efc56aa6732035949537
- **Owner fingerprint:** -
- **Owner since:** 2026-10-25T02:00:15Z
- **Owner until:** 2026-10-25T06:00:15Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T01:32:27Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane R0a 2026-10-25 (read), checked by the lead: __mdh_chunks (runtime/mdh_runtime.c ~9051) computes (len + n - 1) / n and steps i += n / end = i + n in int64; with n near INT64_MAX these overflow (undefined behaviour in C). Repro sketch: chunks([1, 2, 3], 9223372036854775807). The interpreter returns one chunk. Fix: compute the count as len / n + (len % n != 0) and clamp end with a subtraction check.

Evidence fingerprint: `manual:v1:native-chunks-signed-overflow-computing-the-chu-343de05d68adc9e5`


## Fix

<unfixed — raised only>

## Notes

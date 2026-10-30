# MDH-BUG-FLU-00224 — Native log_set_filter does not change the log level

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** runtime
- **Raised:** 2026-10-30T07:06:39Z
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
- **State history:** Open (2026-10-30T07:06:39Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

**Symptom:** natively, `log_set_filter` does not change the log level.

**Repro (0.3.137, observed):**
```
log_set_filter("holler")
blether get_log_level()
```
Interpreter prints `holler`; native prints `blether` (unchanged).

**Where:** runtime/mdh_runtime.c:~5972 (`log_set_filter`).
Found by the docs review, 2026-10-30; reproduced by the lead.

Evidence fingerprint: `manual:v1:native-log-set-filter-does-not-change-the-log-level`


## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00055 — speir truncates input lines at 1023 bytes

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** llvm
- **Raised:** 2026-10-22T23:24:11Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T030536Z-c58fd48f
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00055-run-fix-20261023T030536Z-c58fd48f
- **Owner base:** d31ab2d202d38959050f113496e1e48190cc5322
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:05:36Z
- **Owner until:** 2026-10-23T07:05:36Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred: mdh_runtime.c:1151 fixed buffer; the rest stays in stdin.

## Fix

<unfixed — raised only>

## Notes

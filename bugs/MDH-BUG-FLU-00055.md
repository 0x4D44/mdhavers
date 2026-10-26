# MDH-BUG-FLU-00055 — speir truncates input lines at 1023 bytes

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** llvm
- **Raised:** 2026-10-22T23:24:11Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T171637Z-88f6660f
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00055-run-verify-20261026T171637Z-88f6660f
- **Owner base:** b5fbdef97d31e05c1d4f8b997c745938b368bb11
- **Owner fingerprint:** sha256:78d8c9074f7861d4b47ccc37c3c3b96c08d153ddeabf907a06b7ebf2d488d020
- **Owner since:** 2026-10-26T17:16:37Z
- **Owner until:** 2026-10-26T21:16:37Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:05:42Z, deltic:auto role=fix run=fix-20261023T030536Z-c58fd48f branch=task/bug-MDH-BUG-FLU-00055-run-fix-20261023T030536Z-c58fd48f code=8e071f4 gate=manual)

## Observation

Inferred: mdh_runtime.c:1151 fixed buffer; the rest stays in stdin.

## Fix

<unfixed — raised only>

## Notes

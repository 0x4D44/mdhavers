# MDH-BUG-FLU-00024 — _ rejected as loop variable, lambda or function parameter

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-22T23:24:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T233955Z-12a6ca5e
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00024-run-verify-20261026T233955Z-12a6ca5e
- **Owner base:** 3a7321efd6af1e16d34dcd7fed8e7df5ad38c691
- **Owner fingerprint:** sha256:cd3c71ed1ebd8006bc04f230fb14f4fd189440f1d0aec948c076b90e7afdb61c
- **Owner since:** 2026-10-26T23:39:55Z
- **Owner until:** 2026-10-27T03:39:55Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:52:01Z, deltic:auto role=fix run=fix-20261023T015156Z-7c946d96 branch=task/bug-MDH-BUG-FLU-00024-run-fix-20261023T015156Z-7c946d96 code=8204122 gate=manual)

## Observation

Confirmed: fer _ in 0..3 -> parse error. token.rs:255 / expect_identifier.

## Fix

<unfixed — raised only>

## Notes

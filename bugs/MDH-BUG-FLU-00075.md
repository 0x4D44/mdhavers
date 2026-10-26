# MDH-BUG-FLU-00075 — Newline inside call arguments or before a leading-dot method chain is a parse error

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-23T02:06:58Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T233955Z-0ba3cf7b
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00075-run-verify-20261026T233955Z-0ba3cf7b
- **Owner base:** 3a7321efd6af1e16d34dcd7fed8e7df5ad38c691
- **Owner fingerprint:** sha256:a89ef0705dbbec14943192d21f4a4cd048cfa2227a3b46ada6732c65ff3c1ff5
- **Owner since:** 2026-10-26T23:39:55Z
- **Owner until:** 2026-10-27T03:39:55Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:15:41Z, deltic:auto role=fix run=fix-20261023T031535Z-e74ec7fe branch=task/bug-MDH-BUG-FLU-00075-run-fix-20261023T031535Z-e74ec7fe code=e5fbb29 gate=manual)

## Observation

Found by the showcase lane: splitting a long call's arguments across lines, or starting a continuation line with `.method()`, fails to parse in both backends.

## Fix

<unfixed — raised only>

## Notes

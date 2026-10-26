# MDH-BUG-FLU-00042 — JS f-string text does not escape backslashes

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T225713Z-1f054c82
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00042-run-verify-20261026T225713Z-1f054c82
- **Owner base:** 6cca1b9724e089b14af0018739faf304b5548a5a
- **Owner fingerprint:** sha256:28beaa046c5326234238274a315b4d5659bf27e200cacf0bd1305e5db3d4f144
- **Owner since:** 2026-10-26T22:57:13Z
- **Owner until:** 2026-10-27T02:57:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:17:27Z, deltic:auto role=fix run=fix-20261023T021721Z-4a4d920c branch=task/bug-MDH-BUG-FLU-00042-run-fix-20261023T021721Z-4a4d920c code=3e24904 gate=manual)

## Observation

Confirmed under node: "C:\\users" in an f-string gives a SyntaxError / changed text (compiler.rs:1265-1278).

## Fix

<unfixed — raised only>

## Notes

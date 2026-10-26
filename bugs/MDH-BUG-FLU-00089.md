# MDH-BUG-FLU-00089 — Native float modulo by zero raises a type error instead of divide-by-zero

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T04:31:21Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T171637Z-a685e43e
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00089-run-verify-20261026T171637Z-a685e43e
- **Owner base:** b5fbdef97d31e05c1d4f8b997c745938b368bb11
- **Owner fingerprint:** sha256:c06bcb5b12c0af9da7c78d0a7f010c95f81f97d568fe84a5d1db9ad15e54e46e
- **Owner since:** 2026-10-26T17:16:37Z
- **Owner until:** 2026-10-26T21:16:37Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T04:31:21Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T01:43:21Z, deltic:auto role=fix run=fix-20261024T014315Z-26bc9e7e branch=task/bug-MDH-BUG-FLU-00089-run-fix-20261024T014315Z-26bc9e7e code=5cf1a3f gate=manual)

## Observation

Found by the native hardening lane 2026-10-23: native `1.5 % 0` reports a type error; the interpreter checks the zero right operand before types and reports divide-by-zero.

## Fix

<unfixed — raised only>

## Notes

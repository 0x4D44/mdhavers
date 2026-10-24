# MDH-BUG-FLU-00089 — Native float modulo by zero raises a type error instead of divide-by-zero

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T04:31:21Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T014315Z-26bc9e7e
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00089-run-fix-20261024T014315Z-26bc9e7e
- **Owner base:** 052c51f49463aa5302b2e746fbe4b8cc2542fd41
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T01:43:15Z
- **Owner until:** 2026-10-24T05:43:15Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T04:31:21Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the native hardening lane 2026-10-23: native `1.5 % 0` reports a type error; the interpreter checks the zero right operand before types and reports divide-by-zero.

## Fix

<unfixed — raised only>

## Notes

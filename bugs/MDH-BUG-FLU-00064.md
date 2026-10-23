# MDH-BUG-FLU-00064 — Block expression value is naething unless it uses gie

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** interpreter
- **Raised:** 2026-10-22T23:53:42Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T220141Z-88f4a930
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00064-run-fix-20261023T220141Z-88f4a930
- **Owner base:** 5404d6a092bad9d65c9886f304f8134a819e32d7
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T22:01:41Z
- **Owner until:** 2026-10-24T02:01:41Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:53:42Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Observed by the L3 worker (to confirm intended semantics): `ken a = {0+{0+1}}` parses but fails at runtime with "0 + naething" because a block expression evaluates to naething without an explicit gie (interpreter.rs ~11914). Possibly intended; needs a language decision.

## Fix

<unfixed — raised only>

## Notes

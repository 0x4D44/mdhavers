# MDH-BUG-FLU-00003 — json_parse on deeply nested input aborts the process (unbounded recursion)

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:06Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T014819Z-d15847ec
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00003-run-fix-20261023T014819Z-d15847ec
- **Owner base:** 7b174d476a497248b9f71228cefa2544736a4ccd
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:48:19Z
- **Owner until:** 2026-10-23T05:48:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:06Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: json_parse("[" * 200000) -> stack overflow abort (interpreter.rs JSON parser ~12537-12645 recurses with no depth limit). Should raise a catchable error.

## Fix

<unfixed — raised only>

## Notes

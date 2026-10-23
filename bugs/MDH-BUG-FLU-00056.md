# MDH-BUG-FLU-00056 — stdlib calls functions the interpreter does not define (has_key, slice, tae_json)

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** stdlib
- **Raised:** 2026-10-22T23:24:11Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T024926Z-b94978e1
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00056-run-fix-20261023T024926Z-b94978e1
- **Owner base:** 007e31afdd4ccff8be70811b51a228b3c748f611
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T02:49:26Z
- **Owner until:** 2026-10-23T06:49:26Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred: stdlib/collections.braw:423 group_by calls has_key (interpreter has dict_has; has_key is an LLVM-only alias, and error.rs:643 suggests it); stdlib/uuid.braw:232 slugify calls slice (LLVM-only; interpreter has scran); swarm_symphony/main.braw:55 calls tae_json without fetching json.

## Fix

<unfixed — raised only>

## Notes

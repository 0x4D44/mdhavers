# MDH-BUG-FLU-00010 — Constructors and method calls skip arity checks

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T014937Z-97820486
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00010-run-fix-20261023T014937Z-97820486
- **Owner base:** cb0b93ddad8cd081372ba2d2fdbebf571af72ee9
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:49:37Z
- **Owner until:** 2026-10-23T05:49:37Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: calling a kin constructor or obj.m() with too few args silently binds naething; extra args are dropped (interpreter.rs ~12252, ~11355; call_method_on_instance uses strict params.len() and ignores defaults ~12155).

## Fix

<unfixed — raised only>

## Notes

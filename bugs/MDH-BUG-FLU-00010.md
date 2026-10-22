# MDH-BUG-FLU-00010 — Constructors and method calls skip arity checks

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** -
- **Owner role:** -
- **Owner run:** -
- **Owner host:** -
- **Owner branch:** -
- **Owner base:** -
- **Owner fingerprint:** -
- **Owner since:** -
- **Owner until:** -
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

# MDH-BUG-FLU-00010 — Constructors and method calls skip arity checks

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T224201Z-8c1f301a
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00010-run-verify-20261026T224201Z-8c1f301a
- **Owner base:** 5122a4de097a3cb949560c57a78e74948858cacb
- **Owner fingerprint:** sha256:02b253be2637a54da68bba397b5d0e86cea3fa95971770e640f116a9f04f59c0
- **Owner since:** 2026-10-26T22:42:01Z
- **Owner until:** 2026-10-27T02:42:01Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:49:42Z, deltic:auto role=fix run=fix-20261023T014937Z-97820486 branch=task/bug-MDH-BUG-FLU-00010-run-fix-20261023T014937Z-97820486 code=d84823a gate=manual)

## Observation

Confirmed: calling a kin constructor or obj.m() with too few args silently binds naething; extra args are dropped (interpreter.rs ~12252, ~11355; call_method_on_instance uses strict params.len() and ignores defaults ~12155).

## Fix

<unfixed — raised only>

## Notes

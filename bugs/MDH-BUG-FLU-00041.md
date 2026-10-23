# MDH-BUG-FLU-00041 — JS subclass with its own init crashes (no super call)

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** js
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T021709Z-e540a34b
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00041-run-fix-20261023T021709Z-e540a34b
- **Owner base:** 9a53de7a257cc11471ad68d83664d2417399ace3
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T02:17:09Z
- **Owner until:** 2026-10-23T06:17:09Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed under node: ReferenceError "Must call super constructor" (compiler.rs:705-754). `new` is also chosen by capitalisation of the callee name (:1095), so capitalised functions are constructed and lowercase classes are not.

## Fix

<unfixed — raised only>

## Notes

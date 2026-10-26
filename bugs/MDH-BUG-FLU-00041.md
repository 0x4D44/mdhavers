# MDH-BUG-FLU-00041 — JS subclass with its own init crashes (no super call)

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** js
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-8f0de4ff
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00041-run-verify-20261026T214519Z-8f0de4ff
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:cc126a303e0f5ffeeb2013e543e15caf265bbed361d8cb669ba6a333fe0e8ddc
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:17:15Z, deltic:auto role=fix run=fix-20261023T021709Z-e540a34b branch=task/bug-MDH-BUG-FLU-00041-run-fix-20261023T021709Z-e540a34b code=3e24904 gate=manual)

## Observation

Confirmed under node: ReferenceError "Must call super constructor" (compiler.rs:705-754). `new` is also chosen by capitalisation of the callee name (:1095), so capitalised functions are constructed and lowercase classes are not.

## Fix

<unfixed — raised only>

## Notes

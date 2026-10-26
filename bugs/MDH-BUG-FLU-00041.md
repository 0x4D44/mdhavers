# MDH-BUG-FLU-00041 — JS subclass with its own init crashes (no super call)

- **State:** Closed
- **Priority:** Should
- **Severity:** High
- **Area:** js
- **Raised:** 2026-10-22T23:24:10Z
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
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:17:15Z, deltic:auto role=fix run=fix-20261023T021709Z-e540a34b branch=task/bug-MDH-BUG-FLU-00041-run-fix-20261023T021709Z-e540a34b code=3e24904 gate=manual) -> Closed (2026-10-26T22:40:14Z, deltic:manual role=verify run=verify-20261026T214519Z-8f0de4ff verified_fix_run=fix-20261023T021709Z-e540a34b verdict=close)

## Observation

Confirmed under node: ReferenceError "Must call super constructor" (compiler.rs:705-754). `new` is also chosen by capitalisation of the callee name (:1095), so capitalised functions are constructed and lowercase classes are not.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T22:40:14Z, deltic:manual run=verify-20261026T214519Z-8f0de4ff verified_fix_run=fix-20261023T021709Z-e540a34b verdict=close)

Verifier note: Manual verify: [o1b sol] 3e24904: parity_js_classes_mdh_bug_flu_00041 selected/pass; shared root-hunk mutant: subclass init reverted to constructor without super; parity assertion catches ReferenceError. CLI/Node repro prints Rex!/9/made; restored clean.

## Notes

# MDH-BUG-FLU-00053 — C runtime dict insert copies the whole dict and never frees (quadratic)

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** llvm
- **Raised:** 2026-10-22T23:24:11Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T171637Z-785d6727
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00053-run-verify-20261026T171637Z-785d6727
- **Owner base:** b5fbdef97d31e05c1d4f8b997c745938b368bb11
- **Owner fingerprint:** sha256:1bec32c1a4bcb160c6b5d816f2b517a2961960329977cda818dfcac83fef6cef
- **Owner since:** 2026-10-26T17:16:37Z
- **Owner until:** 2026-10-26T21:16:37Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T04:30:47Z, deltic:auto role=fix run=fix-20261023T043041Z-aa3781ce branch=task/bug-MDH-BUG-FLU-00053-run-fix-20261023T043041Z-aa3781ce code=c9f3fd5 gate=manual)

## Observation

Inferred: mdh_runtime.c:3857-3875 with gc_stub (no free).

## Fix

<unfixed — raised only>

## Notes

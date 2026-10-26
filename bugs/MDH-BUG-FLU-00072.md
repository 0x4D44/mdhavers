# MDH-BUG-FLU-00072 — Native: functions cannot reference globals defined later in the file

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T02:06:58Z
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
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T22:02:02Z, deltic:auto role=fix run=fix-20261023T220155Z-449b24f2 branch=task/bug-MDH-BUG-FLU-00072-run-fix-20261023T220155Z-449b24f2 code=5404d6a gate=manual) -> Closed (2026-10-26T18:20:05Z, deltic:manual role=verify run=verify-20261026T171637Z-1648cdcb verified_fix_run=fix-20261023T220155Z-449b24f2 verdict=close)

## Observation

Found by the showcase lane: a function using a global declared after it fails natively with "Undefined variable: X"; the interpreter resolves it at call time. May be a deliberate restriction; needs a decision and documentation either way.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T18:20:05Z, deltic:manual run=verify-20261026T171637Z-1648cdcb verified_fix_run=fix-20261023T220155Z-449b24f2 verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] recorded 5404d6a is version-only; source fix 58ef56c; function_uses_global_declared_later: 1 passed; nil-forward-load mutant caused add type error at line 6 and test exit assertion failed.

## Notes

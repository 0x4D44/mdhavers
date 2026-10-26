# MDH-BUG-FLU-00178 — Native: a keek match-arm binding leaks out of the arm and overwrites the outer variable

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-26T06:19:59Z
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
- **State history:** Open (2026-10-26T06:19:59Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T09:13:50Z, deltic:auto role=fix run=fix-20261026T065014Z-3a295bf0 branch=task/bug-MDH-BUG-FLU-00178-run-fix-20261026T065014Z-3a295bf0 code=85e470f gate=manual) -> Closed (2026-10-26T21:44:08Z, deltic:manual role=verify run=verify-20261026T194306Z-849e71a8 verified_fix_run=fix-20261026T065014Z-3a295bf0 verdict=close)

## Observation

Found by lane N10 2026-10-26 (code read and probe, src/llvm/codegen.rs compile_match 'Bind pattern variables'): an identifier pattern in a keek arm binds the name in the enclosing scope and is never restored after the arm, so with ken x = 1, a keek arm x => ..., then blether x after the match prints the matched value natively instead of 1 (interpreter). Same family as MDH-BUG-FLU-00163 (block scoping, fixed in N10): give each arm its own scope using the same save/restore mechanism; parity test.

Evidence fingerprint: `manual:v1:native-a-keek-match-arm-binding-leaks-out-of-th-ca0c806941e0513c`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T21:44:08Z, deltic:manual run=verify-20261026T194306Z-849e71a8 verified_fix_run=fix-20261026T065014Z-3a295bf0 verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] 85e470f; six match-scope tests pass. Minimal repro: interpreter 5,1; native 5,1. With arm restore disabled, match_arm_binding_does_not_leak_at_top_level failed output assertion: native [5,5], expected [5,1].

## Notes

# MDH-BUG-FLU-00207 — Native vs interpreter: importer's ken of a fetched module global writes it (native) or shadows it

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-28T15:22:53Z
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
- **State history:** Open (2026-10-28T15:22:53Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-28T19:17:17Z, deltic:auto role=fix run=fix-20261028T174322Z-8fc4cac6 branch=task/bug-MDH-BUG-FLU-00207-run-fix-20261028T174322Z-8fc4cac6 code=f51f4ee gate=manual) -> Closed (2026-10-28T19:29:55Z, deltic:manual role=verify run=verify-20261028T191740Z-6d84dee0 verified_fix_run=fix-20261028T174322Z-8fc4cac6 verdict=close)

## Observation

Native and the interpreter disagree on a top-level `ken` in the importer that names a fetched module global.

Observed by the 205 fix worker (Codex gpt-6.1-sol) on 0.3.106 plus the 205 fix, 2026-10-28; not re-run by the lead.

m.braw:
    ken mg = "mg"
    dae getmg() { gie mg }
main.braw:
    fetch "m"
    ken mg = "new"
    blether getmg()

Native prints `new`: the importer's declaration writes the shared module binding. The interpreter prints `mg`: the declaration makes a new importer binding that shadows the module's.

Expected: one rule on both backends. The DEC doc's round 5 decision (205, share) covers assignment, not redeclaration. The lead's recommendation is the interpreter's (a declaration shadows), but this needs a decision recorded in the DEC doc before a fix.

Evidence fingerprint: `manual:v1:native-vs-interpreter-importer-s-ken-of-a-fetch-c1d98981496c1813`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-28T19:29:55Z, deltic:manual run=verify-20261028T191740Z-6d84dee0 verified_fix_run=fix-20261028T174322Z-8fc4cac6 verdict=close)

Verifier note: Manual verify: Verifier: Codex gpt-6.1-sol xhigh (separate session from the fixer), trunk 23ecbe2. f51f4ee; fetch_bindings_00207 10/10 pass; 207-only mutant: 4 native output assertion failures (top_level_ken new/new vs mg/new). Original repro: mg on interpreter/native/JS. Function, block and nested module ken shadows agree across all three. 207 restored; diff clean before 208 mutant.

## Notes

# MDH-BUG-FLU-00207 — Native vs interpreter: importer's ken of a fetched module global writes it (native) or shadows it

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-28T15:22:53Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261028T191740Z-6d84dee0
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00207-run-verify-20261028T191740Z-6d84dee0
- **Owner base:** c199569ae280800dffce0033bfa39755aeba5385
- **Owner fingerprint:** sha256:be3239374055735d0ef1824180e05e1a68e2286e4ded148b98302c4eb716e7c6
- **Owner since:** 2026-10-28T19:17:40Z
- **Owner until:** 2026-10-28T23:17:40Z
- **Owner session:** flux/2a9090d7-08f7-48c7-bee8-7b7d1d963ab2 (run verify-20261028T191740Z-6d84dee0)
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-28T15:22:53Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-28T19:17:17Z, deltic:auto role=fix run=fix-20261028T174322Z-8fc4cac6 branch=task/bug-MDH-BUG-FLU-00207-run-fix-20261028T174322Z-8fc4cac6 code=f51f4ee gate=manual)

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

## Notes

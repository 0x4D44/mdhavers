# MDH-BUG-FLU-00208 — Native: field write on an aliased module then calling its function gives a type error

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-28T15:22:59Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261028T233512Z-602abe8f
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00208-run-verify-20261028T233512Z-602abe8f
- **Owner base:** d36dafb96ce6381a0a81b3aa270e0a16f6660c0c
- **Owner fingerprint:** sha256:1b397478f8508e0446ae932876bfea8033fddddb310492ebeabf652eacc71e6b
- **Owner since:** 2026-10-28T23:35:12Z
- **Owner until:** 2026-10-29T03:35:12Z
- **Owner session:** flux/2a9090d7-08f7-48c7-bee8-7b7d1d963ab2 (run verify-20261028T233512Z-602abe8f)
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=1, indeterminate=0
- **State history:** Open (2026-10-28T15:22:59Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-28T19:17:23Z, deltic:auto role=fix run=fix-20261028T174254Z-b3b95c5b branch=task/bug-MDH-BUG-FLU-00208-run-fix-20261028T174254Z-b3b95c5b code=f51f4ee gate=manual) -> Open (2026-10-28T19:29:55Z, deltic:manual role=verify run=verify-20261028T191740Z-2678771b verified_fix_run=fix-20261028T174254Z-b3b95c5b verdict=doubt reason=reopen-fail-type-error-in-call-got-function-output-oracle-fails-empty-vs-mg-ok-h) -> Fixed (2026-10-28T22:58:48Z, deltic:auto role=fix run=fix-20261028T214646Z-6e136342 branch=task/bug-MDH-BUG-FLU-00208-run-fix-20261028T214646Z-6e136342 code=96d07e4 gate=manual)

## Observation

Native: writing a field of an aliased module and then calling one of its functions fails with a type error.

Observed by the 205 fix worker (Codex gpt-6.1-sol) on 0.3.106 plus the 205 fix, 2026-10-28; not re-run by the lead.

    fetch "m" as m
    m.mg = "importer"
    blether m.getmg()

where m.braw defines `ken mg` and `dae getmg() { gie mg }`. Native fails with "Type error in 'call': got function". The interpreter keeps aliased imports as a dict of value snapshots, so the call works there and returns the module's own `mg`.

Expected: the call works natively, as on the interpreter. What the field write does to the module's binding is a separate question (aliased imports are snapshots on both backends today).

Evidence fingerprint: `manual:v1:native-field-write-on-an-aliased-module-then-ca-ccf1c1d46c15d9ff`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-28T19:29:55Z, deltic:manual run=verify-20261028T191740Z-2678771b verified_fix_run=fix-20261028T174254Z-b3b95c5b verdict=doubt)

Verifier note: Manual verify: Verifier: Codex gpt-6.1-sol xhigh (separate session from the fixer), trunk 23ecbe2. f51f4ee; fetch_bindings_00208 8/8 pass; 208-only mutant: 4 native assertions fail (Type error in call: got function); output oracle fails empty vs mg/ok!/hi!/importer. Original (tae) prints mg on I[PATH] Residual: m.mg write then m.Box("item") fails native 'Box hasnae been defined yet'; no-write control and I/JS pass. Restored; diff clean. Lead reproduced the residual on 23ecbe2: alias-class.braw native prints "'Box' hasnae been defined yet at line 3"; interpreter prints item:module/module/alias-field; the no-write control passes natively. Same mechanism as the original (alias field write drops the alias's callable targets); the fix covered functions and defaults but not class constructors.

## Notes

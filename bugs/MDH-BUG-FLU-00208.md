# MDH-BUG-FLU-00208 — Native: field write on an aliased module then calling its function gives a type error

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-28T15:22:59Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261028T191740Z-2678771b
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00208-run-verify-20261028T191740Z-2678771b
- **Owner base:** c199569ae280800dffce0033bfa39755aeba5385
- **Owner fingerprint:** sha256:f8977b0536a441c8061ca4cabdb63ac47b74124813a970225ca45157077dfec6
- **Owner since:** 2026-10-28T19:17:40Z
- **Owner until:** 2026-10-28T23:17:40Z
- **Owner session:** flux/2a9090d7-08f7-48c7-bee8-7b7d1d963ab2 (run verify-20261028T191740Z-2678771b)
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-28T15:22:59Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-28T19:17:23Z, deltic:auto role=fix run=fix-20261028T174254Z-b3b95c5b branch=task/bug-MDH-BUG-FLU-00208-run-fix-20261028T174254Z-b3b95c5b code=f51f4ee gate=manual)

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

## Notes

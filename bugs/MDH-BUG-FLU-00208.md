# MDH-BUG-FLU-00208 — Native: field write on an aliased module then calling its function gives a type error

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-28T15:22:59Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261028T174254Z-b3b95c5b
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00208-run-fix-20261028T174254Z-b3b95c5b
- **Owner base:** 3af35a3cbef9fff953b06683d95c320d7a9f483a
- **Owner fingerprint:** -
- **Owner since:** 2026-10-28T17:42:54Z
- **Owner until:** 2026-10-28T21:42:54Z
- **Owner session:** flux/2a9090d7-08f7-48c7-bee8-7b7d1d963ab2 (run fix-20261028T174254Z-b3b95c5b)
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-28T15:22:59Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

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

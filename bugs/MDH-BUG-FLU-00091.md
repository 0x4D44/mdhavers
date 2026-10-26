# MDH-BUG-FLU-00091 — Native dict inline reads scan linearly; dict remove and heave_oot copy the whole dict

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T04:31:22Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261026T214549Z-4263463c
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00091-run-fix-20261026T214549Z-4263463c
- **Owner base:** e9aacdb5b2f66f7b00214d225728b201140c07cf
- **Owner fingerprint:** -
- **Owner since:** 2026-10-26T21:45:49Z
- **Owner until:** 2026-10-27T01:45:49Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=1, indeterminate=0
- **State history:** Open (2026-10-23T04:31:22Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T22:24:50Z, deltic:auto role=fix run=fix-20261024T213341Z-844c3ee0 branch=task/bug-MDH-BUG-FLU-00091-run-fix-20261024T213341Z-844c3ee0 code=3f5bb72 gate=manual) -> Open (2026-10-26T18:20:05Z, deltic:manual role=verify run=verify-20261026T171637Z-87d7f591 verified_fix_run=fix-20261024T213341Z-844c3ee0 verdict=doubt reason=reopen-failed-the-10s-assertion-twice-pre-67-dict-remove-reinsertion-mutant-left)

## Observation

Found by the native hardening lane 2026-10-23: inserts are now amortised O(1) with a hash index (MDH-BUG-FLU-00053), but compile_dict_index's inline read path still scans entries, and dict_remove / heave_oot still rebuild the dict per call (quadratic for remove-heavy loops).

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T18:20:05Z, deltic:manual run=verify-20261026T171637Z-87d7f591 verified_fix_run=fix-20261024T213341Z-844c3ee0 verdict=doubt)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] 67a0617/3f5bb72; all three 00091 tests pass on trunk; pre-67 linear-scan and copy-heave_oot mutants each failed the 10s assertion twice; pre-67 dict_remove reinsertion mutant left two_hundred_thousand_dict_reads_finish_quickly_and_dict_remove_copies green (1 passed): test does not cover dict_remove fix.

## Notes

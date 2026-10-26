# MDH-BUG-FLU-00185 — Native: wrong-type errors from stoater, dicht, is_a and chynge differ from the interpreter

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-26T09:14:16Z
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
- **State history:** Open (2026-10-26T09:14:16Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T11:36:23Z, deltic:auto role=fix run=fix-20261026T091430Z-dedd0d69 branch=task/bug-MDH-BUG-FLU-00185-run-fix-20261026T091430Z-dedd0d69 code=aa422ef gate=manual) -> Closed (2026-10-26T21:44:08Z, deltic:manual role=verify run=verify-20261026T194306Z-b53fb7b7 verified_fix_run=fix-20261026T091430Z-dedd0d69 verdict=close)

## Observation

Found by lane S25 2026-10-26 (executed while fixing MDH-BUG-FLU-00176): with a non-list or wrong-type argument, native stoater, dicht and is_a print the generic native type error (e.g. Och! Type error in 'dicht': got integer) where the interpreter says e.g. dicht() needs a list; native chynge's out-of-range error (runtime/mdh_runtime.c __mdh_chynge, idx > len check) lacks the 'Jings! Something went awfy wrang: ' prefix the interpreter uses. Align the texts with src/interpreter.rs; native-vs-interpreter parity tests.

Evidence fingerprint: `manual:v1:native-wrong-type-errors-from-stoater-dicht-is--748ccf209d96a868`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T21:44:08Z, deltic:manual run=verify-20261026T194306Z-b53fb7b7 verified_fix_run=fix-20261026T091430Z-dedd0d69 verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] aa422ef; 5 parity tests selected/pass, including wrong_type_arguments_have_the_interpreters_text, chynge_out_of_range_has_the_interpreters_text, and is_a_with_a_non_string_type_has_the_interpreters_text; stoater mutant failed native-vs-interpreter assertion (generic type error vs Scots text); old chynge range hunk failed missing Jings prefix; native/interpreter repro matched.

## Notes

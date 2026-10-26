# MDH-BUG-FLU-00185 — Native: wrong-type errors from stoater, dicht, is_a and chynge differ from the interpreter

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-26T09:14:16Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T194306Z-b53fb7b7
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00185-run-verify-20261026T194306Z-b53fb7b7
- **Owner base:** 36750bc8afd262a99873992d919d9ed250946aef
- **Owner fingerprint:** sha256:f630f8d1862e9c60c0f44e36a31164bb255e58c4012175374004d0ee96428081
- **Owner since:** 2026-10-26T19:43:06Z
- **Owner until:** 2026-10-26T23:43:06Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T09:14:16Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T11:36:23Z, deltic:auto role=fix run=fix-20261026T091430Z-dedd0d69 branch=task/bug-MDH-BUG-FLU-00185-run-fix-20261026T091430Z-dedd0d69 code=aa422ef gate=manual)

## Observation

Found by lane S25 2026-10-26 (executed while fixing MDH-BUG-FLU-00176): with a non-list or wrong-type argument, native stoater, dicht and is_a print the generic native type error (e.g. Och! Type error in 'dicht': got integer) where the interpreter says e.g. dicht() needs a list; native chynge's out-of-range error (runtime/mdh_runtime.c __mdh_chynge, idx > len check) lacks the 'Jings! Something went awfy wrang: ' prefix the interpreter uses. Align the texts with src/interpreter.rs; native-vs-interpreter parity tests.

Evidence fingerprint: `manual:v1:native-wrong-type-errors-from-stoater-dicht-is--748ccf209d96a868`


## Fix

<unfixed — raised only>

## Notes

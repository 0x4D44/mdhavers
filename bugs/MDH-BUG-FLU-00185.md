# MDH-BUG-FLU-00185 — Native: wrong-type errors from stoater, dicht, is_a and chynge differ from the interpreter

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-26T09:14:16Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261026T091430Z-dedd0d69
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00185-run-fix-20261026T091430Z-dedd0d69
- **Owner base:** 2779582158f149cb2371904e6f78bcaa51abc16e
- **Owner fingerprint:** -
- **Owner since:** 2026-10-26T09:14:30Z
- **Owner until:** 2026-10-26T13:14:30Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T09:14:16Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane S25 2026-10-26 (executed while fixing MDH-BUG-FLU-00176): with a non-list or wrong-type argument, native stoater, dicht and is_a print the generic native type error (e.g. Och! Type error in 'dicht': got integer) where the interpreter says e.g. dicht() needs a list; native chynge's out-of-range error (runtime/mdh_runtime.c __mdh_chynge, idx > len check) lacks the 'Jings! Something went awfy wrang: ' prefix the interpreter uses. Align the texts with src/interpreter.rs; native-vs-interpreter parity tests.

Evidence fingerprint: `manual:v1:native-wrong-type-errors-from-stoater-dicht-is--748ccf209d96a868`


## Fix

<unfixed — raised only>

## Notes

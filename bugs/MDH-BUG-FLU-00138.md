# MDH-BUG-FLU-00138 — Native rejects an undefined variable at compile time; the interpreter raises it at run time

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-25T06:33:48Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261025T184736Z-08b4a753
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00138-run-fix-20261025T184736Z-08b4a753
- **Owner base:** 19f63d42cb01741fd9cedfd526714c5914eb3ae3
- **Owner fingerprint:** -
- **Owner since:** 2026-10-25T18:47:36Z
- **Owner until:** 2026-10-26T02:10:50Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T06:33:48Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane R1 2026-10-25, reproduced by the lead on trunk 769e76d: examples/features/try_catch.braw uses some_undefined_var inside hae_a_bash to demonstrate catching the error; the interpreter runs it (exit 0) but mdhavers build fails "Compilation went tits up: Undefined variable: some_undefined_var". Parity question: either native compiles an undefined name to a runtime hurl (catchable), or this is documented as a native compile-time check and the example is marked SKIP_NATIVE.

Evidence fingerprint: `manual:v1:native-rejects-an-undefined-variable-at-compile-1dfa75eddec1ce80`


## Fix

<unfixed — raised only>

## Notes

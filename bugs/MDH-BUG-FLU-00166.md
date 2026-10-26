# MDH-BUG-FLU-00166 — Native: calling a function with too many arguments crashes the LLVM verifier

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-26T05:03:31Z
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
- **State history:** Open (2026-10-26T05:03:31Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T06:50:02Z, deltic:auto role=fix run=fix-20261026T050440Z-116dd0d2 branch=task/bug-MDH-BUG-FLU-00166-run-fix-20261026T050440Z-116dd0d2 code=0f7096c gate=manual)

## Observation

Found by lane H1 2026-10-26, reproduced by the lead: `dae f(a) { gie a }` / `blether f(1, 2)`: the interpreter raises "Function 'f' expects 1 arguments but ye gave it 2"; `mdhavers build` fails with "Module verification failed: Incorrect number of arguments passed to called function!" (an internal error, not a user error). JS silently ignores the extra argument; WASM rejects it clearly at compile time. Fix: check arity of direct calls to known functions in src/llvm/codegen.rs and report a Scots compile error naming the function, expected and given counts (or a runtime error if that is the native convention for this case, cf. bug 148); test too many and too few.

Evidence fingerprint: `manual:v1:native-calling-a-function-with-too-many-argumen-45f304f2014105b9`


## Fix

<unfixed — raised only>

## Notes

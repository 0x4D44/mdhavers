# MDH-BUG-FLU-00173 — stacktrace() and error texts differ by backend; JS join() accepts a number separator

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-26T05:04:04Z
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
- **State history:** Open (2026-10-26T05:04:04Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane H1 2026-10-26 (executed): stacktrace() prints the frames in the interpreter, "<stacktrace not available>" natively, "(no stack trace)" in WASM, and does not exist in JS. JS hae_a_bash handlers see JavaScript's own exception text (not the interpreter's Scots message) when the error comes from a native JS exception. `join([3,1,2], 2)` is an error in the interpreter but returns "32122" in JS. Fix the join type check in the JS runtime; document or align stacktrace and error texts.

Evidence fingerprint: `manual:v1:stacktrace-and-error-texts-differ-by-backend-js-2c72050788ccf564`


## Fix

<unfixed — raised only>

## Notes

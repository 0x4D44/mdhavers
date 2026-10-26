# MDH-BUG-FLU-00173 — stacktrace() and error texts differ by backend; JS join() accepts a number separator

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-26T05:04:04Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T225713Z-6848fec3
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00173-run-verify-20261026T225713Z-6848fec3
- **Owner base:** 6cca1b9724e089b14af0018739faf304b5548a5a
- **Owner fingerprint:** sha256:6d181fcf60d85972ca9ef988e644a5f2cbe26ebc90d27296d810d04326546e27
- **Owner since:** 2026-10-26T22:57:13Z
- **Owner until:** 2026-10-27T02:57:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T05:04:04Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T05:56:40Z, deltic:auto role=fix run=fix-20261026T050440Z-84a06621 branch=task/bug-MDH-BUG-FLU-00173-run-fix-20261026T050440Z-84a06621 code=b1616f5 gate=manual)

## Observation

Found by lane H1 2026-10-26 (executed): stacktrace() prints the frames in the interpreter, "<stacktrace not available>" natively, "(no stack trace)" in WASM, and does not exist in JS. JS hae_a_bash handlers see JavaScript's own exception text (not the interpreter's Scots message) when the error comes from a native JS exception. `join([3,1,2], 2)` is an error in the interpreter but returns "32122" in JS. Fix the join type check in the JS runtime; document or align stacktrace and error texts.

Evidence fingerprint: `manual:v1:stacktrace-and-error-texts-differ-by-backend-js-2c72050788ccf564`


## Fix

<unfixed — raised only>

## Notes

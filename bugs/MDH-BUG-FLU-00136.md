# MDH-BUG-FLU-00136 — speir at end of input returns "" (same as an empty line), so input loops spin forever on EOF

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-25T06:26:36Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T224201Z-a2d07549
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00136-run-verify-20261026T224201Z-a2d07549
- **Owner base:** 5122a4de097a3cb949560c57a78e74948858cacb
- **Owner fingerprint:** sha256:e7a93184355ef539df7c1449abc7ac214c1eba4fc709f6890c96e5e24902fe3a
- **Owner since:** 2026-10-26T22:42:01Z
- **Owner until:** 2026-10-27T02:42:01Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T06:26:36Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T23:04:08Z, deltic:auto role=fix run=fix-20261025T184736Z-68f41e51 branch=task/bug-MDH-BUG-FLU-00136-run-fix-20261025T184736Z-68f41e51 code=1ae54b6 gate=manual)

## Observation

Found by lane R1 2026-10-25 (executed; its corpus run filled 29 GB on FLUX when interactive games looped on closed stdin), interpreter path checked by the lead: src/interpreter.rs ~12516 calls read_line and ignores the Ok(0) EOF case, returning Value::String(""). Native behaves the same (per R1). A program cannot tell EOF from an empty line, so any "keep asking until valid input" loop never ends when stdin closes. Needs a language decision from Arthur: at EOF should speir return naething, raise a catchable error, or keep "" and expose an eof() builtin? Then apply on all backends.

Evidence fingerprint: `manual:v1:speir-at-end-of-input-returns-same-as-an-empty--a0ad63c635b4e879`


## Fix

<unfixed — raised only>

## Notes

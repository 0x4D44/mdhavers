# MDH-BUG-FLU-00136 — speir at end of input returns "" (same as an empty line), so input loops spin forever on EOF

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-25T06:26:36Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261025T184736Z-68f41e51
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00136-run-fix-20261025T184736Z-68f41e51
- **Owner base:** 19f63d42cb01741fd9cedfd526714c5914eb3ae3
- **Owner fingerprint:** -
- **Owner since:** 2026-10-25T18:47:36Z
- **Owner until:** 2026-10-25T22:47:36Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T06:26:36Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane R1 2026-10-25 (executed; its corpus run filled 29 GB on FLUX when interactive games looped on closed stdin), interpreter path checked by the lead: src/interpreter.rs ~12516 calls read_line and ignores the Ok(0) EOF case, returning Value::String(""). Native behaves the same (per R1). A program cannot tell EOF from an empty line, so any "keep asking until valid input" loop never ends when stdin closes. Needs a language decision from Arthur: at EOF should speir return naething, raise a catchable error, or keep "" and expose an eof() builtin? Then apply on all backends.

Evidence fingerprint: `manual:v1:speir-at-end-of-input-returns-same-as-an-empty--a0ad63c635b4e879`


## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00120 — WASM: arithmetic and builtin errors carry no line number

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T04:08:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261027T001425Z-e1704d34
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00120-run-verify-20261027T001425Z-e1704d34
- **Owner base:** bb18b71792fa6c1bc5f55d254da1f174c2d0af01
- **Owner fingerprint:** sha256:51422c46914bff80792731ed30be093372c7ec2d1752532d2d56b7cb375968d5
- **Owner since:** 2026-10-27T00:14:25Z
- **Owner until:** 2026-10-27T04:14:25Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T04:08:10Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T05:06:35Z, deltic:auto role=fix run=fix-20261024T050629Z-f77df4c3 branch=task/bug-MDH-BUG-FLU-00120-run-fix-20261024T050629Z-f77df4c3 code=cce0f83 gate=manual)

## Observation

Found by lane W3 (937d1ff) 2026-10-24: host-side errors from arithmetic (e.g. overflow) and builtin calls omit the interpreter's 'at line N', so a caught-and-printed message differs (tests/golden/basics/operators/integer_model.braw is in tests/wasm_validity_allowlist.txt for this). Pass the statement line to the host as the native backend does with __mdh_current_line.

Evidence fingerprint: `manual:v1:wasm-arithmetic-and-builtin-errors-carry-no-line-number`


## Fix

<unfixed — raised only>

## Notes

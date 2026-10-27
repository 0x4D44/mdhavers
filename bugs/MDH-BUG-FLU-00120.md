# MDH-BUG-FLU-00120 — WASM: arithmetic and builtin errors carry no line number

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T04:08:10Z
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
- **State history:** Open (2026-10-24T04:08:10Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T05:06:35Z, deltic:auto role=fix run=fix-20261024T050629Z-f77df4c3 branch=task/bug-MDH-BUG-FLU-00120-run-fix-20261024T050629Z-f77df4c3 code=cce0f83 gate=manual) -> Closed (2026-10-27T01:32:14Z, deltic:manual role=verify run=verify-20261027T001425Z-e1704d34 verified_fix_run=fix-20261024T050629Z-f77df4c3 verdict=close)

## Observation

Found by lane W3 (937d1ff) 2026-10-24: host-side errors from arithmetic (e.g. overflow) and builtin calls omit the interpreter's 'at line N', so a caught-and-printed message differs (tests/golden/basics/operators/integer_model.braw is in tests/wasm_validity_allowlist.txt for this). Pass the statement line to the host as the native backend does with __mdh_current_line.

Evidence fingerprint: `manual:v1:wasm-arithmetic-and-builtin-errors-carry-no-line-number`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:32:14Z, deltic:manual run=verify-20261027T001425Z-e1704d34 verified_fix_run=fix-20261024T050629Z-f77df4c3 verdict=close)

Verifier note: Manual verify: [ld luna] fix e0c5eb1; host_errors_carry_the_statement_line passed with lines 5/16/22/27/33; guest_line=0 mutant failed WASM/interpreter parity on missing line numbers.

## Notes

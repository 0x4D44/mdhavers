# MDH-BUG-FLU-00120 — WASM: arithmetic and builtin errors carry no line number

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T04:08:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T050629Z-f77df4c3
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00120-run-fix-20261024T050629Z-f77df4c3
- **Owner base:** cce0f83a6af9cb894a2cd55fc6347fa73242fa63
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T05:06:29Z
- **Owner until:** 2026-10-24T09:06:29Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T04:08:10Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane W3 (937d1ff) 2026-10-24: host-side errors from arithmetic (e.g. overflow) and builtin calls omit the interpreter's 'at line N', so a caught-and-printed message differs (tests/golden/basics/operators/integer_model.braw is in tests/wasm_validity_allowlist.txt for this). Pass the statement line to the host as the native backend does with __mdh_current_line.

Evidence fingerprint: `manual:v1:wasm-arithmetic-and-builtin-errors-carry-no-line-number`


## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00075 — Newline inside call arguments or before a leading-dot method chain is a parse error

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-23T02:06:58Z
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
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:15:41Z, deltic:auto role=fix run=fix-20261023T031535Z-e74ec7fe branch=task/bug-MDH-BUG-FLU-00075-run-fix-20261023T031535Z-e74ec7fe code=e5fbb29 gate=manual) -> Closed (2026-10-27T01:51:48Z, deltic:manual role=verify run=verify-20261026T233955Z-0ba3cf7b verified_fix_run=fix-20261023T031535Z-e74ec7fe verdict=close)

## Observation

Found by the showcase lane: splitting a long call's arguments across lines, or starting a continuation line with `.method()`, fails to parse in both backends.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:51:48Z, deltic:manual run=verify-20261026T233955Z-0ba3cf7b verified_fix_run=fix-20261023T031535Z-e74ec7fe verdict=close)

Verifier note: Manual verify: [lc luna] e5fbb29: parser_followups and JS parity test passed; removing newline skips failed `call_arguments_may_span_lines` and `leading_dot_continues_a_method_chain` with `Unexpected token` diagnostics. Source diff restored clean.

## Notes

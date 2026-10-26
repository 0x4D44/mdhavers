# MDH-BUG-FLU-00064 — Block expression value is naething unless it uses gie

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** interpreter
- **Raised:** 2026-10-22T23:53:42Z
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
- **State history:** Open (2026-10-22T23:53:42Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T22:01:48Z, deltic:auto role=fix run=fix-20261023T220141Z-88f4a930 branch=task/bug-MDH-BUG-FLU-00064-run-fix-20261023T220141Z-88f4a930 code=5404d6a gate=manual) -> Closed (2026-10-26T23:48:13Z, deltic:manual role=verify run=verify-20261026T224201Z-b6dd00b4 verified_fix_run=fix-20261023T220141Z-88f4a930 verdict=close)

## Observation

Observed by the L3 worker (to confirm intended semantics): `ken a = {0+{0+1}}` parses but fails at runtime with "0 + naething" because a block expression evaluates to naething without an explicit gie (interpreter.rs ~11914). Possibly intended; needs a language decision.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:48:13Z, deltic:manual run=verify-20261026T224201Z-b6dd00b4 verified_fix_run=fix-20261023T220141Z-88f4a930 verdict=close)

Verifier note: Manual verify: [la luna] 955d38a; parity_js_semantics_decisions_f1_f2_f3_block_value selected 1 pass. Removing `value = v` made it fail with interpreter `0 + naething`, reproducing the record; baseline output includes `1`. Restored; source diff clean.

## Notes

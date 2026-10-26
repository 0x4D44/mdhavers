# MDH-BUG-FLU-00037 — JS compiler emits match string patterns unescaped (code injection)

- **State:** Closed
- **Priority:** Should
- **Severity:** High
- **Area:** js
- **Raised:** 2026-10-22T23:24:10Z
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
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:16:29Z, deltic:auto role=fix run=fix-20261023T021624Z-0103fadb branch=task/bug-MDH-BUG-FLU-00037-run-fix-20261023T021624Z-0103fadb code=3e24904 gate=manual) -> Closed (2026-10-26T22:40:14Z, deltic:manual role=verify run=verify-20261026T214519Z-1340c377 verified_fix_run=fix-20261023T021624Z-0103fadb verdict=close)

## Observation

Confirmed under node: a match arm string containing `" || console.log("INJECTED") || "` executes. compiler.rs:959 format!("\"{}\"", s) without escaping.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T22:40:14Z, deltic:manual run=verify-20261026T214519Z-1340c377 verified_fix_run=fix-20261023T021624Z-0103fadb verdict=close)

Verifier note: Manual verify: [o1b sol] 3e24904: parity_js_string_escaping_mdh_bug_flu_00037_00038_00042 selected/pass; shared root-hunk mutant: unescaped match literal executes INJECTED; parity assertion rejects output. CLI/Node repro prints only other; restored clean.

## Notes

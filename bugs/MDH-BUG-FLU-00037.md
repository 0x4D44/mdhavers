# MDH-BUG-FLU-00037 — JS compiler emits match string patterns unescaped (code injection)

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** js
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T021624Z-0103fadb
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00037-run-fix-20261023T021624Z-0103fadb
- **Owner base:** d7df6f3bec718529648445329f7bb765dbc49f14
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T02:16:24Z
- **Owner until:** 2026-10-23T06:16:24Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed under node: a match arm string containing `" || console.log("INJECTED") || "` executes. compiler.rs:959 format!("\"{}\"", s) without escaping.

## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00037 — JS compiler emits match string patterns unescaped (code injection)

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** js
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-1340c377
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00037-run-verify-20261026T214519Z-1340c377
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:38ac5e55b72b5c4ae4a147062669e5c79f2ec587d83f5a6feb00f7a43422ac73
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:16:29Z, deltic:auto role=fix run=fix-20261023T021624Z-0103fadb branch=task/bug-MDH-BUG-FLU-00037-run-fix-20261023T021624Z-0103fadb code=3e24904 gate=manual)

## Observation

Confirmed under node: a match arm string containing `" || console.log("INJECTED") || "` executes. compiler.rs:959 format!("\"{}\"", s) without escaping.

## Fix

<unfixed — raised only>

## Notes

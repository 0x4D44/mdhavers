# MDH-BUG-FLU-00125 — WASM: forward-global reads, block ken over globals, non-constant dae defaults

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T05:29:44Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261027T001425Z-20077765
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00125-run-verify-20261027T001425Z-20077765
- **Owner base:** bb18b71792fa6c1bc5f55d254da1f174c2d0af01
- **Owner fingerprint:** sha256:6fb4b6e79427512b3345a6fb9fb2bc2ca8bc5df502c3b19438616641de0363e1
- **Owner since:** 2026-10-27T00:14:25Z
- **Owner until:** 2026-10-27T04:14:25Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T05:29:44Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T06:14:48Z, deltic:auto role=fix run=fix-20261024T061442Z-c872c15f branch=task/bug-MDH-BUG-FLU-00125-run-fix-20261024T061442Z-c872c15f code=c436abd gate=manual)

## Observation

Leftovers documented by lane W6 (06978e5, wrk_docs/2026.10.24 - DES - WASM classes and globals.md) 2026-10-24: (1) a dae reading a global before its ken has run gets naething where the interpreter raises 'hasnae been defined' (same class as native MDH-BUG-FLU-00095); (2) a top-level fer or block ken reusing a global's name writes the global instead of making a fresh binding; (3) top-level and nested daes cannot take non-constant defaults (the method argc mechanism would allow it; clears ~4 allowlist entries); (4) a kin that is not a top-level statement is a compile error; (5) operator-overload calls do not restore the statement line afterwards.

Evidence fingerprint: `manual:v1:wasm-forward-global-reads-block-ken-over-global-76ea018f4e8a983f`


## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00165 — Interpreter recursion overflows at about 700-900 calls deep in a default-features macOS build

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** interpreter
- **Raised:** 2026-10-26T05:03:26Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261026T050440Z-c0058d8e
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00165-run-fix-20261026T050440Z-c0058d8e
- **Owner base:** 0ce2a5bc5cf9ba91c13f5b84148dcd8cfea0ba99
- **Owner fingerprint:** -
- **Owner since:** 2026-10-26T05:04:40Z
- **Owner until:** 2026-10-26T09:04:40Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T05:03:26Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane H1 2026-10-26 (debug build, default features incl. graphics): `dae r(n) { gin n == 0 { gie 0 } gie 1 + r(n - 1) }` / `blether r(900)` exits 1 with the stack error; r(600) works. Native, JS and WASM handle 4999. Inference from H1: src/main.rs CLI_STACK_SIZE (the big interpreter thread stack) is not applied on macOS when the graphics feature is on (raylib wants the main thread), so the default 8 MB main stack applies. Check release too. Fix: keep the big stack in that configuration (e.g. run the interpreter on a big-stack thread unless the program actually opens a window), and a test for depth 5000 under default features.

Evidence fingerprint: `manual:v1:interpreter-recursion-overflows-at-about-700-90-f5e0670bd7a135dc`


## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00165 — Interpreter recursion overflows at about 700-900 calls deep in a default-features macOS build

- **State:** Closed
- **Priority:** Should
- **Severity:** High
- **Area:** interpreter
- **Raised:** 2026-10-26T05:03:26Z
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
- **State history:** Open (2026-10-26T05:03:26Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T06:25:12Z, deltic:auto role=fix run=fix-20261026T050440Z-c0058d8e branch=task/bug-MDH-BUG-FLU-00165-run-fix-20261026T050440Z-c0058d8e code=eaf1af8 gate=manual) -> Closed (2026-10-26T22:40:14Z, deltic:manual role=verify run=verify-20261026T214519Z-1e9b3021 verified_fix_run=fix-20261026T050440Z-c0058d8e verdict=close)

## Observation

Found by lane H1 2026-10-26 (debug build, default features incl. graphics): `dae r(n) { gin n == 0 { gie 0 } gie 1 + r(n - 1) }` / `blether r(900)` exits 1 with the stack error; r(600) works. Native, JS and WASM handle 4999. Inference from H1: src/main.rs CLI_STACK_SIZE (the big interpreter thread stack) is not applied on macOS when the graphics feature is on (raylib wants the main thread), so the default 8 MB main stack applies. Check release too. Fix: keep the big stack in that configuration (e.g. run the interpreter on a big-stack thread unless the program actually opens a window), and a test for depth 5000 under default features.

Evidence fingerprint: `manual:v1:interpreter-recursion-overflows-at-about-700-90-f5e0670bd7a135dc`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T22:40:14Z, deltic:manual run=verify-20261026T214519Z-1e9b3021 verified_fix_run=fix-20261026T050440Z-c0058d8e verdict=close)

Verifier note: Manual verify: [o1b sol] eaf1af8: default graphics macOS cli_run_recursion_5000_deep_succeeds 1 passed; reverting linker stack to 8MiB fails success assertion with stack error. Original r(900) prints 900; 512MiB linker setting applies all profiles; restored clean.

## Notes

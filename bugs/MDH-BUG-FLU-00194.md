# MDH-BUG-FLU-00194 — cargo build --all-targets fails: wasm_tak_drap_strings uses wasm_runner without a feature gate

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** tests
- **Raised:** 2026-10-26T19:24:00Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261026T214549Z-9b7fd66c
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00194-run-fix-20261026T214549Z-9b7fd66c
- **Owner base:** e9aacdb5b2f66f7b00214d225728b201140c07cf
- **Owner fingerprint:** -
- **Owner since:** 2026-10-26T21:45:49Z
- **Owner until:** 2026-10-27T01:45:49Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T19:24:00Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane R5 2026-10-26 (executed): tests/wasm_tak_drap_strings.rs:6 uses mdhavers::wasm_runner without #![cfg(feature = "wasm_runner")], so cargo build --all-targets with default features fails with E0432 (unresolved import). Gate the file like the other wasm_* tests (lead added this file in 11d6d03).

Evidence fingerprint: `manual:v1:cargo-build-all-targets-fails-wasm-tak-drap-str-8fd0ca838fc53f40`


## Fix

<unfixed — raised only>

## Notes

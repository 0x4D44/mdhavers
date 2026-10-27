# MDH-BUG-FLU-00194 — cargo build --all-targets fails: wasm_tak_drap_strings uses wasm_runner without a feature gate

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** tests
- **Raised:** 2026-10-26T19:24:00Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261027T013251Z-19a47685
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00194-run-verify-20261027T013251Z-19a47685
- **Owner base:** dc207b08e001e4c28ffbe641571b9bde4a9b16c3
- **Owner fingerprint:** sha256:dbf4198acf66e3b7c92db527f70b40f19d18b3f18bc13d38cf83f331f3aa767c
- **Owner since:** 2026-10-27T01:32:51Z
- **Owner until:** 2026-10-27T05:32:51Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T19:24:00Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-27T00:56:30Z, deltic:auto role=fix run=fix-20261026T214549Z-9b7fd66c branch=task/bug-MDH-BUG-FLU-00194-run-fix-20261026T214549Z-9b7fd66c code=094641b gate=manual)

## Observation

Found by lane R5 2026-10-26 (executed): tests/wasm_tak_drap_strings.rs:6 uses mdhavers::wasm_runner without #![cfg(feature = "wasm_runner")], so cargo build --all-targets with default features fails with E0432 (unresolved import). Gate the file like the other wasm_* tests (lead added this file in 11d6d03).

Evidence fingerprint: `manual:v1:cargo-build-all-targets-fails-wasm-tak-drap-str-8fd0ca838fc53f40`


## Fix

<unfixed — raised only>

## Notes

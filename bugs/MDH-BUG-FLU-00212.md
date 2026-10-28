# MDH-BUG-FLU-00212 — cargo clippy -D warnings fails on trunk with --features cli,llvm,native

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** tooling
- **Raised:** 2026-10-28T21:34:47Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261028T233537Z-4d4508ec
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00212-run-verify-20261028T233537Z-4d4508ec
- **Owner base:** de9e4f57b7673b679b4c1bdd6ef7ebd7d263965e
- **Owner fingerprint:** sha256:2b67cf0cadd764d9c654eb3e3eb943d869fd6cfcbd7e6895b66f48c5ce09feed
- **Owner since:** 2026-10-28T23:35:37Z
- **Owner until:** 2026-10-29T03:35:37Z
- **Owner session:** flux/2a9090d7-08f7-48c7-bee8-7b7d1d963ab2 (run verify-20261028T233537Z-4d4508ec)
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-28T21:34:47Z, raised via `deltic bugs new --land`) -> Fixed (2026-10-28T21:46:06Z, deltic:auto role=fix run=fix-20261028T214559Z-4c457540 branch=task/bug-MDH-BUG-FLU-00212-run-fix-20261028T214559Z-4c457540 code=df7a6e0 gate=manual)

## Observation

## Observation

`cargo clippy --no-default-features --features cli,llvm,native -- -D warnings` (the CLAUDE.md "Code Quality" command, with the lean feature set the workers use) fails on trunk.

Seen independently by two Codex Sol workers on 2026-10-28 (Bogle stage 1 on trunk 23ecbe2-era; stage 8b on 0.3.113), both in code their lanes did not touch:

1. runtime/mdh_runtime_rs/src/lib.rs:112, :118: `RustFrames` is never constructed and `enter` is never used when neither `audio` nor `graphics3d` is enabled (dead_code under -D warnings). This stops clippy before it reaches the main crate.
2. With that bypassed, 27 further lints in the main crate, including: src/compiler.rs:903 (`map_or(true, ..)` -> `is_none_or`), src/interpreter.rs:2653 (needless borrow), :7639 (collapsible match), :12003/:12006/:12914/:12919/:12925 (needless borrows), src/interpreter.rs:1721/:1730 (`dict_get_bytes`/`dict_get_u16` unused under `native`+unix), src/wasm_compiler.rs:894/:922/:996, src/llvm/codegen/rc.rs:1774 (redundant closure), and older src/llvm/codegen.rs helpers (char-as-u8 casts, `map` that should be `inspect`, `or_insert_with(HashMap::new)`, needless range loops, a trailing `()`).

Expected: clean, so `make clippy` and lane clippy gates are usable.

Likely cause (inference): newer clippy lints (toolchain update) plus cfg combinations not covered when the code was written (lean features without audio/graphics3d).

Evidence fingerprint: `manual:v1:cargo-clippy-d-warnings-fails-on-trunk-with-fea-8060e8d1a226afcd`


## Fix

<unfixed — raised only>

## Notes

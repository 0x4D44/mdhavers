# Lessons learnt

- tests/golden_tests.rs:run_interpreter_test - debug builds use ~45 KB of stack per mdhavers call; run interpreter
  programs on a big-stack thread (the CLI uses 512 MiB). On a 2 MB test thread the StackOverflow guard fires after a few dozen calls (2026-10-23).
- .cargo/config.toml - it is tracked (aliases). For vendored/offline builds pass source replacement with `--config`
  or via CARGO_HOME; a written `.cargo/config.toml` is silently restored by `git checkout -f` (2026-10-23).
- playground/Cargo.lock - a separate lockfile for the path-dependent playground crate; keep it in sync
  (`cargo update -p mdhavers --offline` in playground/) or the integration version bump fails `cargo metadata --locked` (2026-10-23).

- build.rs:main - cc >= 1.1 names objects `<hash>-<stem>.o`; never `include_bytes!` a bare `OUT_DIR/<stem>.o`.
  The LLVM runtime embed only "worked" from a stale OUT_DIR, and the platform layer was never embedded.
  Embed the static archive that `Build::compile` produces instead (2026-10-23).
- Cargo.toml:inkwell - the llvm feature needs LLVM 15 exactly. On macOS: `brew install llvm@15` (keg-only),
  then `LLVM_SYS_150_PREFIX=/opt/homebrew/opt/llvm@15 cargo build`. Homebrew's default `llvm` (23) does not work (2026-10-23).
- src/interpreter.rs:resolve_ipv4_addr - BSD sockaddrs have `sin_len` and a u8 family; use `libc::sa_family_t`,
  never `as u16`. The suite had only ever been run on Linux `full`; run `--features minimal` too (2026-10-23).

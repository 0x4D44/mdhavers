# Lessons learnt

- build.rs:main - cc >= 1.1 names objects `<hash>-<stem>.o`; never `include_bytes!` a bare `OUT_DIR/<stem>.o`.
  The LLVM runtime embed only "worked" from a stale OUT_DIR, and the platform layer was never embedded.
  Embed the static archive that `Build::compile` produces instead (2026-10-23).
- Cargo.toml:inkwell - the llvm feature needs LLVM 15 exactly. On macOS: `brew install llvm@15` (keg-only),
  then `LLVM_SYS_150_PREFIX=/opt/homebrew/opt/llvm@15 cargo build`. Homebrew's default `llvm` (23) does not work (2026-10-23).
- src/interpreter.rs:resolve_ipv4_addr - BSD sockaddrs have `sin_len` and a u8 family; use `libc::sa_family_t`,
  never `as u16`. The suite had only ever been run on Linux `full`; run `--features minimal` too (2026-10-23).

# Lessons learnt

- stdlib/maths.braw:range_step - an agent's repro of a suspected infinite loop outlived its shell: the parent was
  killed, the mdhavers child was re-parented to launchd and grew to 41 GB, forcing a site-wide build hold on FLUX
  (2026-10-30, MDH-BUG-FLU-00248). Run every hang repro as `deltic timeout N` directly on the mdhavers binary (not on
  a wrapper shell) with a memory cap (`ulimit -v`), and say so in worker briefs that ask for repros.
- games/bogle/art/src:PROMPTS.md - a Codex worker that generates images through the deltic codex pool keeps every
  image in its session history, so after about ten 2 MB images the request passes the pool's 32 MiB cap (HTTP 413) and
  the thread cannot be resumed. Run image generation as a chain of fresh sessions of at most five images each
  (.scratch/w28/art-chain.sh pattern), shrink to the committed size before review, and state chroma-key tolerances
  numerically: a brief saying "pure #FF00FF" made five sessions reject every good sprite (backgrounds ~245,10,245).
- tests/interpreter_dict_remove_perf.rs:heave_oot_removal_costs_a_small_multiple_of_a_plain_loop - on the shared FLUX
  Mac (load 60-250) a perf test must compare against a same-size control loop timed in the same process, not a
  wall-clock limit and not a ratio across sizes: even a plain interpreter loop there scales superlinearly with size
  (13 ms at 5k -> 150-360 ms at 40k), so a linear heave_oot measured 10x for 4x keys. Two trunk-red flakes on
  2026-10-27 (198, 202). Size the limit so trunk and the quadratic mutant are each >=10x away.
- Makefile:coverage - `--cfg coverage` tests (`*_coverage_only.rs`, `src/llvm/coverage_tests.rs`) compile in no
  focused gate, so semantic changes break them silently (the coverage build was broken 2026-10-23 to 2026-10-25). After a
  language-semantics or Value-layout change, run `RUSTFLAGS="--cfg coverage" cargo test --no-default-features --features
  cli,llvm,native --lib --tests --no-fail-fast` with its own CARGO_TARGET_DIR under `.scratch/`.
- src/parser.rs:return_statement - a change to what the parser accepts breaks inline .braw in any test file; grep
  every tests/*.rs (llvm_*, *_coverage*, wasm_*) for the construct, or run the full suite, before landing. Two trunk
  breaks on 2026-10-24 came from top-level `{ gie .. }` sources in native and WASM tests no golden run touched.
- tests/*_parity_allowlist.txt - the JS and WASM golden harnesses fail when an allowlisted golden starts passing, so a
  lane that fixes a builtin must rebase onto the latest goldens and prune the list before landing (2026-10-24).
- git:squash - never squash a lane with `git reset --soft origin/master` after a fetch: if origin moved, the commit
  silently reverts every file other lanes landed meanwhile. Use `git reset --soft $(git merge-base HEAD origin/master)`,
  then check `git diff --stat origin/master` lists only your files. Three lanes slipped this way on 2026-10-23 (all caught).
- src/value.rs:Drop for Value - Value has a Drop impl (iterative teardown of deep containers), so matching a Value by
  value and moving a field out is E0509; match on `&value` and clone the field (2026-10-23).

- tests/golden_tests.rs:run_interpreter_test - debug builds use ~45 KB of stack per mdhavers call; run interpreter
  programs on a big-stack thread (the CLI uses 512 MiB). On a 2 MB test thread the StackOverflow guard fires after a few dozen calls (2026-10-23).
- .cargo/config.toml - it is tracked (aliases). For vendored/offline builds pass source replacement with `--config`
  or via CARGO_HOME; a written `.cargo/config.toml` is silently restored by `git checkout -f` (2026-10-23).
- playground/Cargo.lock - a separate lockfile for the path-dependent playground crate; keep it in sync
  (`cargo update -p mdhavers --offline` in playground/) or the integration version bump fails `cargo metadata --locked` (2026-10-23).

- build.rs:main - cc >= 1.1 names objects `<hash>-<stem>.o`; never `include_bytes!` a bare `OUT_DIR/<stem>.o`.
  The LLVM runtime embed only "worked" from a stale OUT_DIR, and the platform layer was never embedded.
  Embed the static archive that `Build::compile` produces instead (2026-10-23).
- Cargo.toml:inkwell - the llvm feature needs LLVM 19 exactly (`llvm19-1`, since U4 2026-10-25; was 15). On macOS:
  `brew install llvm@19` (keg-only), then `LLVM_SYS_191_PREFIX=/opt/homebrew/opt/llvm@19 cargo build`. Homebrew's
  default `llvm` (23) does not work; the env var name tracks the llvm-sys major (191), not the LLVM version (2026-10-23).
- src/interpreter.rs:resolve_ipv4_addr - BSD sockaddrs have `sin_len` and a u8 family; use `libc::sa_family_t`,
  never `as u16`. The suite had only ever been run on Linux `full`; run `--features minimal` too (2026-10-23).

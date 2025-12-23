# Vendored Dependencies

This directory contains third-party code vendored into the repository.

## `raylib-sys-5.5.1/`

`mdhavers` uses raylib (via the `raylib` crate) for the optional `graphics` feature.
The low-level `raylib-sys` crate is pinned to a vendored copy via:

- `Cargo.toml` → `[patch.crates-io]` → `raylib-sys = { path = "vendor/raylib-sys-5.5.1" }`

### Why it is vendored

- To pin a known-good `raylib-sys` tree (including its bundled `raylib/` sources and build script behavior) for reproducible builds.

### Updating

If you need to update `raylib-sys`:

1. Pick the desired upstream `raylib-sys` version.
2. Replace `vendor/raylib-sys-*/` with the new contents.
3. Update `Cargo.toml` to point `[patch.crates-io] raylib-sys` at the new vendored path.
4. Build and test with graphics enabled:
   - `cargo test --no-default-features --features cli,native,graphics`

### Security / provenance notes

- Vendored code should be treated as an extension of the project’s trusted computing base.
- Prefer small, explicit diffs when updating vendor trees (and review upstream changelogs/licenses as part of the update).

# mdhavers WASM assets

- `mdh_rustysynth.wasm` is a tiny Rust WebAssembly module that wraps `rustysynth`
  for MIDI rendering in the JavaScript/WASM backends.

## Rebuild

```bash
cd runtime/mdh_rustysynth_wasm
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/mdh_rustysynth_wasm.wasm ../../assets/wasm/mdh_rustysynth.wasm
```

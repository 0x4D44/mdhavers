# MDH-BUG-FLU-00183 — Native runtime: Rust code raises via longjmp across live Rust frames (tri, audio)

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-26T08:39:16Z
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
- **State history:** Open (2026-10-26T08:39:16Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane N10 2026-10-26 (code audit while fixing MDH-BUG-FLU-00164; not executed): the Rust runtime calls C functions that raise by longjmp while Rust values are alive, which skips their destructors (UB per the Rust Reference; at least leaks). Sites: __mdh_hurl at runtime/mdh_runtime_rs/src/audio.rs:437 and audio_stub.rs:12; about ten __mdh_hurl sites in tri_runtime.rs, some with a live format!() String (~1440/1460/1488); __mdh_key_not_found (tri_runtime.rs:1500); __mdh_list_get, __mdh_dict_set and __mdh_eq calls in tri_runtime.rs that raise on bad input. Fix: raise only at the extern entry after Rust values are dropped (pattern: mdh_value_to_string_or_raise from the 164 fix), or return a status across FFI.

Evidence fingerprint: `manual:v1:native-runtime-rust-code-raises-via-longjmp-acr-2385b56230ad04d9`


## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00145 — Native has about 410 builtin names the interpreter lacks, many with placeholder bodies

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T08:13:34Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T182021Z-8a8d9f68
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00145-run-verify-20261026T182021Z-8a8d9f68
- **Owner base:** 2f352f49ef4c71b45a0497bf81a04779c622faad
- **Owner fingerprint:** sha256:9bf81d3e21949f313907a7f976744bb480f0b547e163a055a22d46f945294a4a
- **Owner since:** 2026-10-26T18:20:21Z
- **Owner until:** 2026-10-26T22:20:21Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T08:13:34Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T23:21:06Z, deltic:auto role=fix run=fix-20261025T223241Z-2be50da0 branch=task/bug-MDH-BUG-FLU-00145-run-fix-20261025T223241Z-2be50da0 code=e820711 gate=manual)

## Observation

Found by lane S10's builtin audit 2026-10-25: 410 names in the native compile_call dispatch are undefined in the interpreter, and many have placeholder bodies returning nil, an empty string or the first argument (e.g. matrix_*, curry, partial, memoize, cache, constantly, once, throttle, debounce, pipe, event_*, store_*, table_*, config_*, cli_*, http_*, test_suite/describe/it, gen_*/forall/property, uuid, base64_*, md5/sha256, url_encode/decode, slugify/camel_case/snake_case, insert_at, deep_copy, dict_pop, try_catch/throw/raise, char_code_at, times/unless/case/switch, key_down/key_pressed). A program using one gets silently wrong results natively and "undefined" in the interpreter. Needs Arthur's decision: remove them from native, or implement the wanted ones on every backend.

Evidence fingerprint: `manual:v1:native-has-about-410-builtin-names-the-interpre-104a9348e0a83a6a`


## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00145 — Native has about 410 builtin names the interpreter lacks, many with placeholder bodies

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T08:13:34Z
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
- **State history:** Open (2026-10-25T08:13:34Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T23:21:06Z, deltic:auto role=fix run=fix-20261025T223241Z-2be50da0 branch=task/bug-MDH-BUG-FLU-00145-run-fix-20261025T223241Z-2be50da0 code=e820711 gate=manual) -> Closed (2026-10-26T19:40:44Z, deltic:manual role=verify run=verify-20261026T182021Z-8a8d9f68 verified_fix_run=fix-20261025T223241Z-2be50da0 verdict=close)

## Observation

Found by lane S10's builtin audit 2026-10-25: 410 names in the native compile_call dispatch are undefined in the interpreter, and many have placeholder bodies returning nil, an empty string or the first argument (e.g. matrix_*, curry, partial, memoize, cache, constantly, once, throttle, debounce, pipe, event_*, store_*, table_*, config_*, cli_*, http_*, test_suite/describe/it, gen_*/forall/property, uuid, base64_*, md5/sha256, url_encode/decode, slugify/camel_case/snake_case, insert_at, deep_copy, dict_pop, try_catch/throw/raise, char_code_at, times/unless/case/switch, key_down/key_pressed). A program using one gets silently wrong results natively and "undefined" in the interpreter. Needs Arthur's decision: remove them from native, or implement the wanted ones on every backend.

Evidence fingerprint: `manual:v1:native-has-about-410-builtin-names-the-interpre-104a9348e0a83a6a`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T19:40:44Z, deltic:manual run=verify-20261026T182021Z-8a8d9f68 verified_fix_run=fix-20261025T223241Z-2be50da0 verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] fix=e820711; matrix_new repro is undefined in interpreter and native compile; all 3 llvm_native_placeholder_builtins_removed tests passed; mutant reintroduced curry=>nil and removed_placeholder_builtins_are_undefined failed expect_err: call compiled.

## Notes

# MDH-BUG-FLU-00114 — JS runtime lacks most interpreter builtins (about 240 names by a regex scan)

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-24T03:01:40Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T225713Z-390c33cb
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00114-run-verify-20261026T225713Z-390c33cb
- **Owner base:** 6cca1b9724e089b14af0018739faf304b5548a5a
- **Owner fingerprint:** sha256:b24f0e8e23127fd2560cbcec8f728961e1fc27ad05d786da58be375b74aceb3c
- **Owner since:** 2026-10-26T22:57:13Z
- **Owner until:** 2026-10-27T02:57:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T03:01:40Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T22:08:28Z, deltic:auto role=fix run=fix-20261024T213341Z-379570d4 branch=task/bug-MDH-BUG-FLU-00114-run-fix-20261024T213341Z-379570d4 code=380bacf gate=manual)

## Observation

Found by lane J5 2026-10-24 with a regex comparison of src/interpreter.rs define_natives against RUNTIME_BINDINGS in src/compiler.rs: 367 interpreter names vs 167 JS bindings. Missing include common string/list helpers (chars, trim, words, zip, enumerate, flatten, uniq, starts_wi, pad_left, json_parse, json_pretty, regex_*), date/time, bytes, bit ops, assert_*, file I/O, plus categories that may not belong in JS at all (sockets, tls/dtls/srtp, threads/channels/mutexes, shell, env). The list is approximate (a few test-only names; inline-emitted helpers may be missed). Needs a decision on which categories JS should support, then porting in batches; the JS golden allowlist shows which ones real programs hit first (e.g. chars for showcase_blether_lisp).

Evidence fingerprint: `manual:v1:js-runtime-lacks-most-interpreter-builtins-abou-fd13e24199738a6c`


## Fix

<unfixed — raised only>

## Notes

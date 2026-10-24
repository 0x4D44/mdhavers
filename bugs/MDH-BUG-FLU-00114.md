# MDH-BUG-FLU-00114 — JS runtime lacks most interpreter builtins (about 240 names by a regex scan)

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-24T03:01:40Z
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
- **State history:** Open (2026-10-24T03:01:40Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane J5 2026-10-24 with a regex comparison of src/interpreter.rs define_natives against RUNTIME_BINDINGS in src/compiler.rs: 367 interpreter names vs 167 JS bindings. Missing include common string/list helpers (chars, trim, words, zip, enumerate, flatten, uniq, starts_wi, pad_left, json_parse, json_pretty, regex_*), date/time, bytes, bit ops, assert_*, file I/O, plus categories that may not belong in JS at all (sockets, tls/dtls/srtp, threads/channels/mutexes, shell, env). The list is approximate (a few test-only names; inline-emitted helpers may be missed). Needs a decision on which categories JS should support, then porting in batches; the JS golden allowlist shows which ones real programs hit first (e.g. chars for showcase_blether_lisp).

Evidence fingerprint: `manual:v1:js-runtime-lacks-most-interpreter-builtins-abou-fd13e24199738a6c`


## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00206 — JS backend cannot fetch a .braw source module (require of a package, no exports)

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-28T15:22:45Z
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
- **State history:** Open (2026-10-28T15:22:45Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-28T19:14:38Z, deltic:auto role=fix run=fix-20261028T174332Z-547d5301 branch=task/bug-MDH-BUG-FLU-00206-run-fix-20261028T174332Z-547d5301 code=1da6da1 gate=manual) -> Closed (2026-10-28T19:29:55Z, deltic:manual role=verify run=verify-20261028T191740Z-c9ca35fd verified_fix_run=fix-20261028T174332Z-547d5301 verdict=close)

## Observation

The JS backend cannot import a `.braw` source module with `fetch`.

Observed by the 205 fix worker (Codex gpt-6.1-sol) on trunk 0.3.106, 2026-10-28, and reported to the lead; not re-run by the lead.

- `mdhavers compile main.braw` where main.braw does `fetch "m"` emits `foreign(require(path))` (src/compiler.rs:1670), so Node looks for an npm package called `m`: "Cannot find module 'm'".
- Compiling m.braw to a sibling m.js and writing `fetch "./m"` lets Node find it, but it fails with "ReferenceError: mg is not defined": compiled `.braw` files export nothing, and the importer gets only the alias or basename.
- The file compiler calls `compile(source)` with no source-file context (src/main.rs:429).

Expected: `fetch` of a `.braw` module works in JS as on the interpreter and native, with globals shared per the round 5 decision (DEC doc, 205). The repro from MDH-BUG-FLU-00205 should print `mgxy` then `nh`.

This needs design before a fix: how JS output is laid out for multi-file programs (one bundle, or one file per module).

Evidence fingerprint: `manual:v1:js-backend-cannot-fetch-a-braw-source-module-re-73c242e276b37894`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-28T19:29:55Z, deltic:manual run=verify-20261028T191740Z-c9ca35fd verified_fix_run=fix-20261028T174332Z-547d5301 verdict=close)

Verifier note: Manual verify: Verifier: Codex gpt-6.1-sol xhigh (separate session from the fixer), trunk 23ecbe2. 1da6da1; js_module_fetch 22/22 pass; old compile(&source) mutant 22 fail, missing_source_module_error_is_catchable_and_matches_interpreter output assertion: Cannot find module vs Cannae find module. Original205 interpreter/standalone Node: mgxy/nh after .braw deletion. Restored; diff clean.

## Notes

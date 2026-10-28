# MDH-BUG-FLU-00206 — JS backend cannot fetch a .braw source module (require of a package, no exports)

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-28T15:22:45Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261028T174332Z-547d5301
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00206-run-fix-20261028T174332Z-547d5301
- **Owner base:** 937ffae8461dfb340ba7c4e3fd9fc3da3c924fa2
- **Owner fingerprint:** -
- **Owner since:** 2026-10-28T17:43:32Z
- **Owner until:** 2026-10-28T21:43:32Z
- **Owner session:** flux/2a9090d7-08f7-48c7-bee8-7b7d1d963ab2 (run fix-20261028T174332Z-547d5301)
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-28T15:22:45Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

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

## Notes

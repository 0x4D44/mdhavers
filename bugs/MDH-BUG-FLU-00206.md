# MDH-BUG-FLU-00206 — JS backend cannot fetch a .braw source module (require of a package, no exports)

- **State:** Open
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

# MDH-BUG-FLU-00217 — Native RC: destructuring a string with a rest pattern releases a dead string

- **State:** Open
- **Priority:** Must
- **Severity:** High
- **Area:** native
- **Raised:** 2026-10-29T15:20:26Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261029T152036Z-da5d7908
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00217-run-fix-20261029T152036Z-da5d7908
- **Owner base:** 089d2413a26eb6e759ffb1c794b8a017fb9e4877
- **Owner fingerprint:** -
- **Owner since:** 2026-10-29T15:20:36Z
- **Owner until:** 2026-10-29T19:20:36Z
- **Owner session:** flux/2a9090d7-08f7-48c7-bee8-7b7d1d963ab2 (run fix-20261029T152036Z-da5d7908)
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-29T15:20:26Z, raised via `deltic bugs new --land`)

## Observation

## Observation

With reference counting on (`MDH_RC=1` today, the planned default), destructuring a string with a rest pattern aborts at a later use of the bound variables: the runtime reports a release of a dead string object. List destructuring with the same pattern is fine, and RC off prints the right output.

Found by the RC default-flip worker: `tests/llvm_destructure_string.rs` `llvm_destructure_string_supports_rest_pattern` fails once RC is on by default (0.3.127 base, macOS arm64).

## Repro

```
ken [first, ...mid, last] = "hello"
blether first
blether len(mid)
blether last
```

`MDH_RC=1 mdhavers build r.braw -o r && ./r`

## Expected vs actual

- Expected: `h`, `3`, `o` (RC off prints this).
- `MDH_RC=1`: prints `h`, `3`, `o` but stderr has `mdhavers rc: release o' a deid string object 0x... (it dee'd at line 0), at line 4`, and the program aborts.
- The multi-byte string `"hełło"` (from the test) fails the same way, at line 5.
- `ken [first, ...mid, last] = ["h","e","l","l","o"]` under `MDH_RC=1`: correct, no abort.

## Pointers (not a diagnosis)

The string path of destructuring (split into one-character strings, then bind) is the only difference from the list case. The dead object "dee'd at line 0", which suggests a compiler-generated release rather than one at a source line.

Evidence fingerprint: `manual:v1:native-rc-destructuring-a-string-with-a-rest-pa-5ececf41990001c2`


## Fix

<unfixed — raised only>

## Notes

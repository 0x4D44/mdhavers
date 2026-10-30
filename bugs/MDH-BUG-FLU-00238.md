# MDH-BUG-FLU-00238 — http url_encode and url_decode are stubs

- **State:** Open
- **Priority:** Could
- **Severity:** Medium
- **Area:** stdlib
- **Raised:** 2026-10-30T07:15:08Z
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
- **State history:** Open (2026-10-30T07:15:08Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

`url_encode("a b&c=d/é")` gives `a+b&c=d/é`; `url_decode("a+b%26c%3Dd")` gives `a b%26c%3Dd`. Expected percent encoding/decoding. http.braw:379,388.

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:http-url-encode-and-url-decode-are-stubs`


## Fix

<unfixed — raised only>

## Notes

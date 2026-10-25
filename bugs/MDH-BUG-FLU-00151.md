# MDH-BUG-FLU-00151 — C runtime puts declarations right after default: labels, a C23-only construct clang 15 rejects

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-25T09:59:23Z
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
- **State history:** Open (2026-10-25T09:59:23Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane R2 2026-10-25 (executed): runtime/mdh_runtime.c around lines 2206-2207 and 2248-2249 declare a variable immediately after a default: label. That is a C23 extension (current Apple clang warns -Wc23-extensions; clang 15 rejects it), so the runtime does not build with older or stricter compilers. Fix: wrap the case body in braces or move the declaration.

Evidence fingerprint: `manual:v1:c-runtime-puts-declarations-right-after-default-2b70a7bfeca346f7`


## Fix

<unfixed — raised only>

## Notes

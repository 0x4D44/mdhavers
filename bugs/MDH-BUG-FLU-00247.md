# MDH-BUG-FLU-00247 — stdlib modules clash on global names with different signatures

- **State:** Open
- **Priority:** Could
- **Severity:** Low
- **Area:** stdlib
- **Raised:** 2026-10-30T07:15:57Z
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
- **State history:** Open (2026-10-30T07:15:57Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Fetching two modules lets the later definition win silently: `glob_match` is (pattern, text) in patterns_match.braw:12 but (filename, pattern) in patterns.braw:237 (observed: `aye` then `nae` for the same call); `expect_equal` is (actual, expected) in testing.braw:29 but (expected, actual) in test.braw:95; `Table`, `ProgressBar`, `Spinner` defined in both table/cli and menus; scheduler.braw:14 redefines the built-in `tick()` (`fetch "stdlib/scheduler"; blether tick()` prints `1`). Needs a design decision (rename, namespacing or documented precedence).

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:stdlib-modules-clash-on-global-names-with-different-signatures`


## Fix

<unfixed — raised only>

## Notes

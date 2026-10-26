# MDH-BUG-FLU-00176 — Native sclaff flattens only one level; native error texts differ for stoater, dicht, is_a

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-26T05:32:08Z
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
- **State history:** Open (2026-10-26T05:32:08Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T08:27:58Z, deltic:auto role=fix run=fix-20261026T062548Z-f217470f branch=task/bug-MDH-BUG-FLU-00176-run-fix-20261026T062548Z-f217470f code=2a21715 gate=manual) -> Closed (2026-10-26T21:44:08Z, deltic:manual role=verify run=verify-20261026T194306Z-17169f5a verified_fix_run=fix-20261026T062548Z-f217470f verdict=close)

## Observation

Found by lane S23 2026-10-26 (executed): sclaff([1,[2,[3,[4]]],5]) gives [1, 2, [3, [4]], 5] natively but [1, 2, 3, 4, 5] in the interpreter (and now JS). Native error wording also differs from the interpreter for stoater([]), dicht with an out-of-range index and is_a(1, 2). Fix: make native sclaff flatten fully and align those error messages with src/interpreter.rs; parity tests.

Evidence fingerprint: `manual:v1:native-sclaff-flattens-only-one-level-native-er-a6e9d7ad0c975b98`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T21:44:08Z, deltic:manual run=verify-20261026T194306Z-17169f5a verified_fix_run=fix-20261026T062548Z-f217470f verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] 2a21715 + 349da95; 10 parity tests selected/pass; nested repro prints [1, 2, 3, 4, 5] both. Mutants failed sclaff_flattens_every_level with one-level output, stoater/is_a parity assertions on old errors, and dicht_out_of_range with -2 vs wrapped index.

## Notes

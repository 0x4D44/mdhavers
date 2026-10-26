# MDH-BUG-FLU-00176 — Native sclaff flattens only one level; native error texts differ for stoater, dicht, is_a

- **State:** Open
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
- **State history:** Open (2026-10-26T05:32:08Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane S23 2026-10-26 (executed): sclaff([1,[2,[3,[4]]],5]) gives [1, 2, [3, [4]], 5] natively but [1, 2, 3, 4, 5] in the interpreter (and now JS). Native error wording also differs from the interpreter for stoater([]), dicht with an out-of-range index and is_a(1, 2). Fix: make native sclaff flatten fully and align those error messages with src/interpreter.rs; parity tests.

Evidence fingerprint: `manual:v1:native-sclaff-flattens-only-one-level-native-er-a6e9d7ad0c975b98`


## Fix

<unfixed — raised only>

## Notes

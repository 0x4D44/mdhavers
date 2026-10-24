# MDH-BUG-FLU-00113 — JS runtime lacks ilk, grup_up, pairt_by and dict_get/has/remove/invert/fae_pairs builtins

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-24T02:53:39Z
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
- **State history:** Open (2026-10-24T02:53:39Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane J4 2026-10-24: these interpreter builtins are not defined in runtime/js/core_runtime.js, so programs using them fail under the JS backend with ReferenceError. Also the higher-order builtins (sieve, gaun, tumble, ...) throw a raw JS TypeError for a non-list argument where the interpreter says e.g. 'sieve() expects a list as first argument'.

Evidence fingerprint: `manual:v1:js-runtime-lacks-ilk-grup-up-pairt-by-and-dict--3bf929fb35185c46`


## Fix

<unfixed — raised only>

## Notes

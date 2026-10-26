# MDH-BUG-FLU-00113 — JS runtime lacks ilk, grup_up, pairt_by and dict_get/has/remove/invert/fae_pairs builtins

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-24T02:53:39Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T225713Z-92af3b74
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00113-run-verify-20261026T225713Z-92af3b74
- **Owner base:** 6cca1b9724e089b14af0018739faf304b5548a5a
- **Owner fingerprint:** sha256:cbeffbe6926a3a45075b55c5b81745a9981b42c12dc5e24148703fd98b70fe05
- **Owner since:** 2026-10-26T22:57:13Z
- **Owner until:** 2026-10-27T02:57:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T02:53:39Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T03:01:35Z, deltic:auto role=fix run=fix-20261024T030129Z-8660f402 branch=task/bug-MDH-BUG-FLU-00113-run-fix-20261024T030129Z-8660f402 code=b420187 gate=manual)

## Observation

Found by lane J4 2026-10-24: these interpreter builtins are not defined in runtime/js/core_runtime.js, so programs using them fail under the JS backend with ReferenceError. Also the higher-order builtins (sieve, gaun, tumble, ...) throw a raw JS TypeError for a non-list argument where the interpreter says e.g. 'sieve() expects a list as first argument'.

Evidence fingerprint: `manual:v1:js-runtime-lacks-ilk-grup-up-pairt-by-and-dict--3bf929fb35185c46`


## Fix

<unfixed — raised only>

## Notes

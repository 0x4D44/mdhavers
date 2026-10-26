# MDH-BUG-FLU-00005 — String repeat / pad / bytes with huge counts panic with capacity overflow

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:06Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-7981b0fc
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00005-run-verify-20261026T214519Z-7981b0fc
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:b946669309619109b1f20fc86f68d823b0a89a26d7309113825081678b4f784e
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:06Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:48:47Z, deltic:auto role=fix run=fix-20261023T014841Z-de600b67 branch=task/bug-MDH-BUG-FLU-00005-run-fix-20261023T014841Z-de600b67 code=d84823a gate=manual)

## Observation

Confirmed: "ab" * 9223372036854775807 -> panic "capacity overflow" (interpreter.rs ~11997, 6132, 8026). Large non-overflowing counts abort on allocation failure. Should raise a Scots error.

## Fix

<unfixed — raised only>

## Notes

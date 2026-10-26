# MDH-BUG-FLU-00011 — jammy() always returns the same value on macOS (time-seeded one-step LCG)

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T224201Z-c79e960c
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00011-run-verify-20261026T224201Z-c79e960c
- **Owner base:** 5122a4de097a3cb949560c57a78e74948858cacb
- **Owner fingerprint:** sha256:5b343a7a8aa00c55c9b5e876c0324311420cc2c9ae91aa3eec40dc5ff9374d14
- **Owner since:** 2026-10-26T22:42:01Z
- **Owner until:** 2026-10-27T02:42:01Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:49:53Z, deltic:auto role=fix run=fix-20261023T014948Z-4aca2c4a branch=task/bug-MDH-BUG-FLU-00011-run-fix-20261023T014948Z-4aca2c4a code=d84823a gate=manual)

## Observation

Confirmed on macOS: 20 calls of jammy(0, 8) all return 1. The LCG is seeded from wall-clock nanos (microsecond resolution on macOS) and copied 10 times across interpreter.rs (~5977).

## Fix

<unfixed — raised only>

## Notes

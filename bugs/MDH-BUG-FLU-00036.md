# MDH-BUG-FLU-00036 — Log filter rule `net` also matches target `network`

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** logging
- **Raised:** 2026-10-22T23:24:09Z
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
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:49:20Z, deltic:auto role=fix run=fix-20261023T024915Z-2f4d925d branch=task/bug-MDH-BUG-FLU-00036-run-fix-20261023T024915Z-2f4d925d code=77f950a gate=manual) -> Closed (2026-10-26T23:39:42Z, deltic:manual role=verify run=verify-20261026T225713Z-7b796838 verified_fix_run=fix-20261023T024915Z-2f4d925d verdict=close)

## Observation

Confirmed: logging.rs:38 uses a plain starts_with with no segment boundary.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:39:42Z, deltic:manual run=verify-20261026T225713Z-7b796838 verified_fix_run=fix-20261023T024915Z-2f4d925d verdict=close)

Verifier note: Manual verify: [lb luna] 77f950a; test_log_filter_rule_matches_whole_segments_only passed 1/1; mutant restored starts_with and failed at `network`: expected Blether, got Roar. Test also verifies exact net, dotted, and `::` segment matches.

## Notes

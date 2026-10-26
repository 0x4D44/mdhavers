# MDH-BUG-FLU-00060 — Installed mdhavers cannot find its stdlib or prelude

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** install
- **Raised:** 2026-10-22T23:24:12Z
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
- **Attempts:** fix=0, doubt=1, indeterminate=0
- **State history:** Open (2026-10-22T23:24:12Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:50:18Z, deltic:auto role=fix run=fix-20261023T025013Z-3f0327f6 branch=task/bug-MDH-BUG-FLU-00060-run-fix-20261023T025013Z-3f0327f6 code=77f950a gate=manual) -> Open (2026-10-26T23:48:13Z, deltic:manual role=verify run=verify-20261026T224201Z-de2cb897 verified_fix_run=fix-20261023T025013Z-3f0327f6 verdict=doubt reason=reopen-la-luna-77f950a-isolated-bin-lib-fixture-ran-and-loaded-both-installed-pr)

## Observation

Inferred: installer/install.sh puts stdlib in ~/.mdhavers/lib but the resolver looks in <exe dir>/stdlib; MDHAVERS_HOME is never read; a missing prelude is skipped silently (interpreter.rs ~2544).

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:48:13Z, deltic:manual run=verify-20261026T224201Z-de2cb897 verified_fix_run=fix-20261023T025013Z-3f0327f6 verdict=doubt)

Verifier note: Manual verify: [la luna] 77f950a; isolated bin/ + lib/ fixture ran and loaded both installed prelude and module. every_stdlib_module_loads passed before and after removing the installed module search candidates, proving it resolves from the repo and does not cover installation. Restored; source diff clean.

## Notes

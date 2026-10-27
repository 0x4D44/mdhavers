# MDH-BUG-FLU-00060 — Installed mdhavers cannot find its stdlib or prelude

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** install
- **Raised:** 2026-10-22T23:24:12Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261027T045159Z-dcc2e369
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00060-run-verify-20261027T045159Z-dcc2e369
- **Owner base:** b2712adc7b304eb842407eae833911cd438de665
- **Owner fingerprint:** sha256:8bb5ca93dae1145ee1cf85a8218c13b34f2410e8e98c8dea17d44a2c56008610
- **Owner since:** 2026-10-27T04:51:59Z
- **Owner until:** 2026-10-27T08:51:59Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=1, indeterminate=0
- **State history:** Open (2026-10-22T23:24:12Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:50:18Z, deltic:auto role=fix run=fix-20261023T025013Z-3f0327f6 branch=task/bug-MDH-BUG-FLU-00060-run-fix-20261023T025013Z-3f0327f6 code=77f950a gate=manual) -> Open (2026-10-26T23:48:13Z, deltic:manual role=verify run=verify-20261026T224201Z-de2cb897 verified_fix_run=fix-20261023T025013Z-3f0327f6 verdict=doubt reason=reopen-la-luna-77f950a-isolated-bin-lib-fixture-ran-and-loaded-both-installed-pr) -> Fixed (2026-10-27T04:50:29Z, deltic:auto role=fix run=fix-20261027T015252Z-88b79dd5 branch=task/bug-MDH-BUG-FLU-00060-run-fix-20261027T015252Z-88b79dd5 code=aa81b64 gate=manual)

## Observation

Inferred: installer/install.sh puts stdlib in ~/.mdhavers/lib but the resolver looks in <exe dir>/stdlib; MDHAVERS_HOME is never read; a missing prelude is skipped silently (interpreter.rs ~2544).

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:48:13Z, deltic:manual run=verify-20261026T224201Z-de2cb897 verified_fix_run=fix-20261023T025013Z-3f0327f6 verdict=doubt)

Verifier note: Manual verify: [la luna] 77f950a; isolated bin/ + lib/ fixture ran and loaded both installed prelude and module. every_stdlib_module_loads passed before and after removing the installed module search candidates, proving it resolves from the repo and does not cover installation. Restored; source diff clean.

## Notes

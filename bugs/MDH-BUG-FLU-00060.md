# MDH-BUG-FLU-00060 — Installed mdhavers cannot find its stdlib or prelude

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** install
- **Raised:** 2026-10-22T23:24:12Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T025013Z-3f0327f6
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00060-run-fix-20261023T025013Z-3f0327f6
- **Owner base:** 1ee854bf5b76693bc7c2ef4bf9a8419321a78dfa
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T02:50:13Z
- **Owner until:** 2026-10-23T06:50:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:12Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred: installer/install.sh puts stdlib in ~/.mdhavers/lib but the resolver looks in <exe dir>/stdlib; MDHAVERS_HOME is never read; a missing prelude is skipped silently (interpreter.rs ~2544).

## Fix

<unfixed — raised only>

## Notes

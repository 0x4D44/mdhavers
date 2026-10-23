# MDH-BUG-FLU-00059 — Makefile and setup-deps.sh are Linux-only

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** build
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
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:12Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:50:07Z, deltic:auto role=fix run=fix-20261023T025001Z-c9390bf4 branch=task/bug-MDH-BUG-FLU-00059-run-fix-20261023T025001Z-c9390bf4 code=77f950a gate=manual)

## Observation

Inferred: Makefile:10 zstd detection uses /usr/lib/x86_64-linux-gnu so macOS always reports LLVM disabled yet cargo build still builds full; scripts/setup-deps.sh supports apt/dnf/pacman/zypper only.

## Fix

<unfixed — raised only>

## Notes

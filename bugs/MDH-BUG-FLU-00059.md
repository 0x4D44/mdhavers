# MDH-BUG-FLU-00059 — Makefile and setup-deps.sh are Linux-only

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** build
- **Raised:** 2026-10-22T23:24:12Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T025001Z-c9390bf4
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00059-run-fix-20261023T025001Z-c9390bf4
- **Owner base:** c69953cdb09017208cdf38dac271ca0b9fafbe0a
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T02:50:01Z
- **Owner until:** 2026-10-23T06:50:01Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:12Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred: Makefile:10 zstd detection uses /usr/lib/x86_64-linux-gnu so macOS always reports LLVM disabled yet cargo build still builds full; scripts/setup-deps.sh supports apt/dnf/pacman/zypper only.

## Fix

<unfixed — raised only>

## Notes

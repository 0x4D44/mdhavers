# MDH-BUG-FLU-00059 — Makefile and setup-deps.sh are Linux-only

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** build
- **Raised:** 2026-10-22T23:24:12Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T224201Z-7c468c3a
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00059-run-verify-20261026T224201Z-7c468c3a
- **Owner base:** 5122a4de097a3cb949560c57a78e74948858cacb
- **Owner fingerprint:** sha256:d1f015c23976ed674f78d75323078e08ad1ec89dd07b90df2b2db6f1f12bbfa9
- **Owner since:** 2026-10-26T22:42:01Z
- **Owner until:** 2026-10-27T02:42:01Z
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

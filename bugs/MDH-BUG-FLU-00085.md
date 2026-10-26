# MDH-BUG-FLU-00085 — Dropping a very deeply nested list overflows the native stack

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-23T03:06:29Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T224201Z-bffc0426
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00085-run-verify-20261026T224201Z-bffc0426
- **Owner base:** 5122a4de097a3cb949560c57a78e74948858cacb
- **Owner fingerprint:** sha256:2e529c77188a1314cbbf22d6c4892b23f0cd4be372531d8a178928d1a91c789b
- **Owner since:** 2026-10-26T22:42:01Z
- **Owner until:** 2026-10-27T02:42:01Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T03:06:29Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:44:56Z, deltic:auto role=fix run=fix-20261023T034450Z-b0c6280d branch=task/bug-MDH-BUG-FLU-00085-run-fix-20261023T034450Z-b0c6280d code=54979e9 gate=manual)

## Observation

Found by the interpreter stack lane 2026-10-23: `ken a = []` then `fer i in 0..1000000 { a = [a] }` and let the program end; Rc<RefCell<Vec<Value>>> drop recurses per level and can abort the process. Needs an iterative Drop (drain children into a heap work list) for list/dict containers.

## Fix

<unfixed — raised only>

## Notes

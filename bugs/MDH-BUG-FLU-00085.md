# MDH-BUG-FLU-00085 — Dropping a very deeply nested list overflows the native stack

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-23T03:06:29Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T034450Z-b0c6280d
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00085-run-fix-20261023T034450Z-b0c6280d
- **Owner base:** e5b77335bbb4563b5cc3ffc87eecf3977c3d7486
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:44:50Z
- **Owner until:** 2026-10-23T07:44:50Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T03:06:29Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the interpreter stack lane 2026-10-23: `ken a = []` then `fer i in 0..1000000 { a = [a] }` and let the program end; Rc<RefCell<Vec<Value>>> drop recurses per level and can abort the process. Needs an iterative Drop (drain children into a heap work list) for list/dict containers.

## Fix

<unfixed — raised only>

## Notes

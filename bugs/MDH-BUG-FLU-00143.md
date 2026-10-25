# MDH-BUG-FLU-00143 — Native join, split, substr_between and aw segfault on wrong-type arguments

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** native
- **Raised:** 2026-10-25T08:13:26Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261025T081928Z-669e2aee
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00143-run-fix-20261025T081928Z-669e2aee
- **Owner base:** c149176f45c35e9263239f7fe701407d0c4cc731
- **Owner fingerprint:** -
- **Owner since:** 2026-10-25T08:19:28Z
- **Owner until:** 2026-10-25T12:19:28Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T08:13:26Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane S10's native-vs-interpreter builtin audit 2026-10-25 (executed probes, about 11 argument profiles per name): join, split, substr_between and aw crash natively (SIGSEGV) when given a wrong-type argument, where the interpreter raises a type error. Same class as bug 137 (cannie) and S10's first/last fix. Fix: type-check arguments in the native runtime/inline paths and raise the interpreter's error.

Evidence fingerprint: `manual:v1:native-join-split-substr-between-and-aw-segfaul-98104dd831127d38`


## Fix

<unfixed — raised only>

## Notes

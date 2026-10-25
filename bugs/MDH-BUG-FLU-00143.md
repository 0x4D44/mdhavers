# MDH-BUG-FLU-00143 — Native join, split, substr_between and aw segfault on wrong-type arguments

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** native
- **Raised:** 2026-10-25T08:13:26Z
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
- **State history:** Open (2026-10-25T08:13:26Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T09:07:57Z, deltic:auto role=fix run=fix-20261025T081928Z-669e2aee branch=task/bug-MDH-BUG-FLU-00143-run-fix-20261025T081928Z-669e2aee code=1a64677 gate=manual)

## Observation

Found by lane S10's native-vs-interpreter builtin audit 2026-10-25 (executed probes, about 11 argument profiles per name): join, split, substr_between and aw crash natively (SIGSEGV) when given a wrong-type argument, where the interpreter raises a type error. Same class as bug 137 (cannie) and S10's first/last fix. Fix: type-check arguments in the native runtime/inline paths and raise the interpreter's error.

Evidence fingerprint: `manual:v1:native-join-split-substr-between-and-aw-segfaul-98104dd831127d38`


## Fix

<unfixed — raised only>

## Notes

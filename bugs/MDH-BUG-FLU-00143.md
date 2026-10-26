# MDH-BUG-FLU-00143 — Native join, split, substr_between and aw segfault on wrong-type arguments

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** native
- **Raised:** 2026-10-25T08:13:26Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T164142Z-d48c8e31
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00143-run-verify-20261026T164142Z-d48c8e31
- **Owner base:** b5d5a9fa852316d9d5ef58b28e50e2449fbe0bad
- **Owner fingerprint:** sha256:74df1098bbc146b45377533b6e4f272552c90e6413ef4da24d2f0f20d7190da7
- **Owner since:** 2026-10-26T16:41:42Z
- **Owner until:** 2026-10-26T20:41:42Z
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

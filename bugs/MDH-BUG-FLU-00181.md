# MDH-BUG-FLU-00181 — Native on Linux: deep recursion on the main thread still crashes with SIGSEGV

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-26T07:45:19Z
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
- **State history:** Open (2026-10-26T07:45:19Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found in review of MDH-BUG-FLU-00164 (lane N10 and Astra consult, 2026-10-26; source analysis, Linux untested): the stack-guard tripwire is only armed when mprotect really protects the range (runtime/mdh_runtime.c stack guard init). On Linux the main thread's stack bottom is not mapped yet, so mprotect fails and unbounded recursion on the main thread keeps the old SIGSEGV instead of the catchable 'Stack's fair puggled!' error. Small thread stacks are likewise left unprotected. Fix: a checked fallback for the Linux main thread (e.g. run main on a runtime-allocated thread or stack with a real guard, or an SP-vs-rlimit limit check), with a Linux test.

Evidence fingerprint: `manual:v1:native-on-linux-deep-recursion-on-the-main-thre-3774f22f1f66beec`


## Fix

<unfixed — raised only>

## Notes

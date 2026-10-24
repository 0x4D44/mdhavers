# MDH-BUG-FLU-00128 — Native: fer over a dict or creel iterates the handle as a list instead of raising

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-24T22:20:47Z
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
- **State history:** Open (2026-10-24T22:20:47Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane N5 2026-10-24 (executed on the old native build): compile_for_iterable in src/llvm/codegen.rs (~27430) sends every non-string value to compile_for_list_impl, so fer x in a dict or creel reads the handle as a list. Repro: ken c = creel(["a","b","c"])  fer x in c { blether x } -> native prints a float, <object> x6, naething; the interpreter raises "Cannae iterate ower a dict/creel". With tombstoned creels (bug 91 fix) it can read further and crash. Fix: raise the interpreter error for dict/creel handles at run time.

Evidence fingerprint: `manual:v1:native-fer-over-a-dict-or-creel-iterates-the-ha-dd3335514900414a`


## Fix

<unfixed — raised only>

## Notes

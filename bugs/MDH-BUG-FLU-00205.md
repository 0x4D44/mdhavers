# MDH-BUG-FLU-00205 — fetch: module global written by its own function is shared natively but copied in the interpreter

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-27T07:32:29Z
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
- **State history:** Open (2026-10-27T07:32:29Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-28T15:22:09Z, deltic:auto role=fix run=fix-20261028T140936Z-debccb08 branch=task/bug-MDH-BUG-FLU-00205-run-fix-20261028T140936Z-debccb08 code=d45cbd5 gate=manual)

## Observation

The interpreter and native disagree on a module global written by the module's own function after `fetch`.

m.braw:
    ken mg = "m" + "g"
    dae setmg() {
        mg = "n" + "h"
        gie "xy"
    }
main.braw:
    fetch "m"
    blether mg + setmg()
    blether mg

Trunk 0.3.105 (2026-10-27): interpreter prints `mgxy` then `mg`; native (`mdhavers build`, RC off) prints `mgxy` then `nh`. So in the interpreter main's `mg` is a copy taken at fetch time and setmg() writes the module's own binding; natively, main and the module share one binding. Found by lane N16 (MDH-BUG-FLU-00196) while writing an RC probe; that probe's test asserts only the first line for now. JS was not checked (the compiled JS needs the module compiled alongside).

This needs a language decision before a fix: does `fetch` bind the module's globals by value at import time (interpreter) or share them (native)? Check the DEC doc and README for an existing rule, then make every backend match it.

Evidence fingerprint: `manual:v1:fetch-module-global-written-by-its-own-function-436486c3ad89b220`


## Fix

<unfixed — raised only>

## Notes

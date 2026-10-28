# MDH-BUG-FLU-00205 — fetch: module global written by its own function is shared natively but copied in the interpreter

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-27T07:32:29Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261028T152314Z-1711ec7f
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00205-run-verify-20261028T152314Z-1711ec7f
- **Owner base:** c88e1c5950832788cde43a33f5a5e31f8f72d3fa
- **Owner fingerprint:** sha256:9719ec955986136f3a0bbf6a2f917659a36845d9e5102b41c1761fa653404b58
- **Owner since:** 2026-10-28T15:23:14Z
- **Owner until:** 2026-10-28T19:23:14Z
- **Owner session:** flux/2a9090d7-08f7-48c7-bee8-7b7d1d963ab2 (run verify-20261028T152314Z-1711ec7f)
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

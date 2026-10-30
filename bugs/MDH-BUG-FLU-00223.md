# MDH-BUG-FLU-00223 — Native exit() with a float uses raw bits (exit(2.9) gives 51, interpreter 2)

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** llvm
- **Raised:** 2026-10-30T07:06:34Z
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
- **State history:** Open (2026-10-30T07:06:34Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

**Symptom:** `exit` with a float argument exits with a different status natively than in the interpreter.

**Repro (0.3.137, observed):** program `exit(2.9)`. Interpreter: exit status 2. Native (`mdhavers build`): exit status 51.
Expected: the same status on every backend (2, truncation, as the interpreter does).

**Where:** src/llvm/codegen.rs:~26429: the float's raw bits appear to be truncated instead of the value converted.
Found by the docs review, 2026-10-30; reproduced by the lead.

Evidence fingerprint: `manual:v1:native-exit-with-a-float-uses-raw-bits-exit-2-9-e5a56b8fa2f81431`


## Fix

<unfixed — raised only>

## Notes

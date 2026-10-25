# MDH-BUG-FLU-00137 — Native binaries segfault on examples/scottish/scots_fun.braw and scots_words.braw

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** native
- **Raised:** 2026-10-25T06:33:34Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261025T063408Z-d07563ed
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00137-run-fix-20261025T063408Z-d07563ed
- **Owner base:** ca69315e0ea85a61d8fdd5bba98402fdee27d5a9
- **Owner fingerprint:** -
- **Owner since:** 2026-10-25T06:34:08Z
- **Owner until:** 2026-10-25T10:34:08Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T06:33:34Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane R1 2026-10-25, reproduced by the lead on trunk 769e76d (debug compiler, default flags, MDH_RC unset): mdhavers build examples/scottish/scots_fun.braw -o x && ./x </dev/null exits 139 (SIGSEGV) with no output; same for examples/scottish/scots_words.braw. The interpreter runs both to completion (scots_fun prints 79 lines starting "=== Scots Vocabulary Functions ===", first section "haverin (is it nonsense?)"). No stdout reached the pipe, so the crash point is unknown (output may be buffered). Bisect the program to the first crashing builtin.

Evidence fingerprint: `manual:v1:native-binaries-segfault-on-examples-scottish-s-2132e2c005f8f620`


## Fix

<unfixed — raised only>

## Notes

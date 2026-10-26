# MDH-BUG-FLU-00137 — Native binaries segfault on examples/scottish/scots_fun.braw and scots_words.braw

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** native
- **Raised:** 2026-10-25T06:33:34Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T164142Z-4cd9129e
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00137-run-verify-20261026T164142Z-4cd9129e
- **Owner base:** b5d5a9fa852316d9d5ef58b28e50e2449fbe0bad
- **Owner fingerprint:** sha256:af1eaed775a96713a575ea93c05a26764e05a778874c4c48b6728f769d33729b
- **Owner since:** 2026-10-26T16:41:42Z
- **Owner until:** 2026-10-26T20:41:42Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T06:33:34Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T06:54:54Z, deltic:auto role=fix run=fix-20261025T063408Z-d07563ed branch=task/bug-MDH-BUG-FLU-00137-run-fix-20261025T063408Z-d07563ed code=a262039 gate=manual)

## Observation

Found by lane R1 2026-10-25, reproduced by the lead on trunk 769e76d (debug compiler, default flags, MDH_RC unset): mdhavers build examples/scottish/scots_fun.braw -o x && ./x </dev/null exits 139 (SIGSEGV) with no output; same for examples/scottish/scots_words.braw. The interpreter runs both to completion (scots_fun prints 79 lines starting "=== Scots Vocabulary Functions ===", first section "haverin (is it nonsense?)"). No stdout reached the pipe, so the crash point is unknown (output may be buffered). Bisect the program to the first crashing builtin.

Evidence fingerprint: `manual:v1:native-binaries-segfault-on-examples-scottish-s-2132e2c005f8f620`


## Fix

<unfixed — raised only>

## Notes

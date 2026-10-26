# MDH-BUG-FLU-00107 — Native: fer loop variable leaks past the loop and hides the outer binding

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** native
- **Raised:** 2026-10-24T02:38:48Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T164142Z-e287d985
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00107-run-verify-20261026T164142Z-e287d985
- **Owner base:** b5d5a9fa852316d9d5ef58b28e50e2449fbe0bad
- **Owner fingerprint:** sha256:5839fe70c58f8b62f8c01b5d7e449dceb65ea12954ff87df0810ce0eb6f3d99f
- **Owner since:** 2026-10-26T16:41:42Z
- **Owner until:** 2026-10-26T20:41:42Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T02:38:48Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T03:04:07Z, deltic:auto role=fix run=fix-20261024T030402Z-267f96e5 branch=task/bug-MDH-BUG-FLU-00107-run-fix-20261024T030402Z-267f96e5 code=efb4bb2 gate=manual)

## Observation

Found by the lead's review of 5404d6a/58ef56c 2026-10-24 (read, not executed). compile_for_string_impl / compile_for_list_impl / compile_for_range (src/llvm/codegen.rs ~27198, 27357, 27458) insert the loop variable into self.variables and never restore the previous entry; begin_loop_var (~27081) edits boxed_vars without restoring it. Repro A: dae f() { ken x = 10  fer x in [1, 2] { }  gie x }  blether f()  -> interpreter 10, native expected 2. Repro B (nested same name): dae g() { ken fs = []  fer i in 0..2 { fer i in 5..6 { }  shove(fs, || i) }  gie fs[0]() }  -> interpreter 0, native expected 5 or a raw cell. Repro C: ken i = 99  fer i in 0..3 {}  blether i -> interpreter 99. Fix: save/restore variables, boxed_vars, var_types and shadows around each loop.

Evidence fingerprint: `manual:v1:native-fer-loop-variable-leaks-past-the-loop-an-e8c672f0901dcb21`


## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00189 — Native: remaining unchecked allocations (codegen strdup, make_list, empty_dict, slurp/lines/words)

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-26T13:29:44Z
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
- **State history:** Open (2026-10-26T13:29:44Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T16:41:26Z, deltic:auto role=fix run=fix-20261026T140328Z-b8cc2164 branch=task/bug-MDH-BUG-FLU-00189-run-fix-20261026T140328Z-b8cc2164 code=3c19d7e gate=manual) -> Open (2026-10-27T06:15:02Z, deltic:manual role=verify run=verify-20261027T055825Z-5fbfbe41 verified_fix_run=fix-20261026T140328Z-b8cc2164 verdict=doubt reason=reopen-error-and-container-buffers-out-of-memory-raise-the-too-big-errors-failed) -> Fixed (2026-10-27T06:59:02Z, deltic:auto role=fix run=fix-20261027T061545Z-f2c1fbca branch=task/bug-MDH-BUG-FLU-00189-run-fix-20261027T061545Z-f2c1fbca code=039c5b8 gate=manual) -> Closed (2026-10-27T07:05:31Z, deltic:manual role=verify run=verify-20261027T065919Z-2a4e78ea verified_fix_run=fix-20261027T061545Z-f2c1fbca verdict=close)

## Observation

Found by lane N12 2026-10-26 (code read, while fixing MDH-BUG-FLU-00188): codegen calls __mdh_obj_strdup with no NULL check at src/llvm/codegen.rs ~:26005 (getenv), ~:31822 (split's last token) and ~:39520 (object key); __mdh_make_list and __mdh_empty_dict do not check for NULL (called from C and Rust); __mdh_slurp, __mdh_lines and __mdh_words do not check their buffer allocation and leak the temporary buffer. Out-of-memory there writes through NULL instead of raising 'ower muckle'. The codegen sites change IR of common programs, so route them through a checked runtime helper. Note the MDH_ALLOC_FAULTS knob only fails object allocations, not container growth buffers; extend it to test these.

Evidence fingerprint: `manual:v1:native-remaining-unchecked-allocations-codegen--23356c2634d5758b`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T06:15:02Z, deltic:manual run=verify-20261027T055825Z-5fbfbe41 verified_fix_run=fix-20261026T140328Z-b8cc2164 verdict=doubt)

Verifier note: Manual verify: [lh luna] 3c19d7e/8a99f01; four tests passed; compiled_string_copies_raise_the_too_big_error and container_buffers_out_of_memory_raise_the_too_big_errors failed after unchecked allocation mutants, but container_buffers stayed green when mdh_new_list reverted to raising __mdh_make_list across Rust; test misses the no-longjmp fix.

### Verification summary (2026-10-27T07:05:31Z, deltic:manual run=verify-20261027T065919Z-2a4e78ea verified_fix_run=fix-20261027T061545Z-f2c1fbca verdict=close)

Verifier note: Manual verify: [lj luna] 039c5b8; safety 11/11, RC 12/12; empty_dict_out_of_memory_raises_the_too_big_error and slurp_lines_and_words_need_no_temporary_buffer pass; outputs assert TOO_BIG x4, efter, and 2097152 x6. list/dict raising-call mutants make rust_runtime_value_makers_out_of_memory_never_raise_across_rust fail the stderr guard-abort assertion; clean diff after both.

## Notes

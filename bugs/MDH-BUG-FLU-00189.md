# MDH-BUG-FLU-00189 — Native: remaining unchecked allocations (codegen strdup, make_list, empty_dict, slurp/lines/words)

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-26T13:29:44Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261026T140328Z-b8cc2164
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00189-run-fix-20261026T140328Z-b8cc2164
- **Owner base:** 6a3dc52ba0cbde96cbb57922ec28d39ded4a9851
- **Owner fingerprint:** -
- **Owner since:** 2026-10-26T14:03:28Z
- **Owner until:** 2026-10-26T18:03:28Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T13:29:44Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane N12 2026-10-26 (code read, while fixing MDH-BUG-FLU-00188): codegen calls __mdh_obj_strdup with no NULL check at src/llvm/codegen.rs ~:26005 (getenv), ~:31822 (split's last token) and ~:39520 (object key); __mdh_make_list and __mdh_empty_dict do not check for NULL (called from C and Rust); __mdh_slurp, __mdh_lines and __mdh_words do not check their buffer allocation and leak the temporary buffer. Out-of-memory there writes through NULL instead of raising 'ower muckle'. The codegen sites change IR of common programs, so route them through a checked runtime helper. Note the MDH_ALLOC_FAULTS knob only fails object allocations, not container growth buffers; extend it to test these.

Evidence fingerprint: `manual:v1:native-remaining-unchecked-allocations-codegen--23356c2634d5758b`


## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00151 — C runtime puts declarations right after default: labels, a C23-only construct clang 15 rejects

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-25T09:59:23Z
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
- **Attempts:** fix=0, doubt=1, indeterminate=0
- **State history:** Open (2026-10-25T09:59:23Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T10:37:12Z, deltic:auto role=fix run=fix-20261025T100351Z-bed57b91 branch=task/bug-MDH-BUG-FLU-00151-run-fix-20261025T100351Z-bed57b91 code=8497359 gate=manual) -> Open (2026-10-26T21:44:08Z, deltic:manual role=verify run=verify-20261026T194306Z-e22233f2 verified_fix_run=fix-20261025T100351Z-bed57b91 verdict=doubt reason=reopen-verifier-gpt-6-luna-max-codex-recorded-by-lead-8497359-showing-a-float-an)

## Observation

Found by lane R2 2026-10-25 (executed): runtime/mdh_runtime.c around lines 2206-2207 and 2248-2249 declare a variable immediately after a default: label. That is a C23 extension (current Apple clang warns -Wc23-extensions; clang 15 rejects it), so the runtime does not build with older or stricter compilers. Fix: wrap the case body in braces or move the declaration.

Evidence fingerprint: `manual:v1:c-runtime-puts-declarations-right-after-default-2b70a7bfeca346f7`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T21:44:08Z, deltic:manual run=verify-20261026T194306Z-e22233f2 verified_fix_run=fix-20261025T100351Z-bed57b91 verdict=doubt)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] 8497359; showing_a_float_and_splitting_on_a_long_delimiter_leak_nothing selected/pass on trunk and after reversing both C23 braces; clang 21 reports C23-extension warnings under the mutant. Existing test does not cover C labels.

## Notes

# MDH-BUG-FLU-00159 — Native clype on a class instance omits the class name

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-26T02:17:27Z
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
- **State history:** Open (2026-10-26T02:17:27Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T03:29:14Z, deltic:auto role=fix run=fix-20261026T024153Z-5e9d5af4 branch=task/bug-MDH-BUG-FLU-00159-run-fix-20261026T024153Z-5e9d5af4 code=17c24d5 gate=manual) -> Closed (2026-10-26T21:44:08Z, deltic:manual role=verify run=verify-20261026T194306Z-277ea002 verified_fix_run=fix-20261026T024153Z-5e9d5af4 verdict=close)

## Observation

Found by lane S17 2026-10-26 (observed): clype(Dog()) prints '[instance] instance' natively but "[instance] instance o' 'Dog'" in the interpreter. Cause: the default arm of __mdh_clype in runtime/mdh_runtime.c does not look up the instance's class name. Fix: print the class name from the instance header, as tae_string does; parity test.

Evidence fingerprint: `manual:v1:native-clype-on-a-class-instance-omits-the-class-name`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T21:44:08Z, deltic:manual run=verify-20261026T194306Z-277ea002 verified_fix_run=fix-20261026T024153Z-5e9d5af4 verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] 17c24d5; native_clype_names_the_class_of_an_instance passes on trunk. Reversing the MDH_TAG_INSTANCE branch made parity assert fail: native [instance] instance, expected [instance] instance o' 'Dog'.

## Notes

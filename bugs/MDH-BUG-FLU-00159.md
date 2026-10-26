# MDH-BUG-FLU-00159 — Native clype on a class instance omits the class name

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-26T02:17:27Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T194306Z-277ea002
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00159-run-verify-20261026T194306Z-277ea002
- **Owner base:** 36750bc8afd262a99873992d919d9ed250946aef
- **Owner fingerprint:** sha256:9d17a7c6d68d51667cd450afa0b5a6c2194a406ffc501a59810ac4bc019c5f9c
- **Owner since:** 2026-10-26T19:43:06Z
- **Owner until:** 2026-10-26T23:43:06Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T02:17:27Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T03:29:14Z, deltic:auto role=fix run=fix-20261026T024153Z-5e9d5af4 branch=task/bug-MDH-BUG-FLU-00159-run-fix-20261026T024153Z-5e9d5af4 code=17c24d5 gate=manual)

## Observation

Found by lane S17 2026-10-26 (observed): clype(Dog()) prints '[instance] instance' natively but "[instance] instance o' 'Dog'" in the interpreter. Cause: the default arm of __mdh_clype in runtime/mdh_runtime.c does not look up the instance's class name. Fix: print the class name from the instance header, as tae_string does; parity test.

Evidence fingerprint: `manual:v1:native-clype-on-a-class-instance-omits-the-class-name`


## Fix

<unfixed — raised only>

## Notes

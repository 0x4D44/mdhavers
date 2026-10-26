# MDH-BUG-FLU-00144 — Native accepts wrong-type arguments that the interpreter rejects, for about 25 builtins

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T08:13:30Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T182021Z-6ed878b5
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00144-run-verify-20261026T182021Z-6ed878b5
- **Owner base:** 2f352f49ef4c71b45a0497bf81a04779c622faad
- **Owner fingerprint:** sha256:b679c6590c07ae2573143413a8e714c286d15a773f06262215d30d59fcf5a367
- **Owner since:** 2026-10-26T18:20:21Z
- **Owner until:** 2026-10-26T22:20:21Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T08:13:30Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T09:08:04Z, deltic:auto role=fix run=fix-20261025T081928Z-4caa481f branch=task/bug-MDH-BUG-FLU-00144-run-fix-20261025T081928Z-4caa481f code=1a64677 gate=manual)

## Observation

Found by lane S10's builtin audit 2026-10-25 (executed): native silently accepts bad argument types where the interpreter raises a type error: atween, hauld_atween, lerp, gcd, lcm, sign, trunc, is_even, is_odd, is_prime, is_positive, is_negative, is_empty, is_toom, enumerate, starts_with, ends_with, trim, trim_start, trim_end, words, scran; tumble accepts the wrong argument order; hauld_atween("hello","l","L") returns a pointer-sized integer; factorial(2.5) hangs and factorial(-3) returns 1; log_enabled/log_init/log_set_filter/log_span/set_log_level/set_crash_handling ignore invalid input; tls_client_new/tls_connect/tls_close return error dicts where the interpreter raises; scrieve returns aye/nae where the interpreter returns naething.

Evidence fingerprint: `manual:v1:native-accepts-wrong-type-arguments-that-the-in-2b6c6cbf3864729c`


## Fix

<unfixed — raised only>

## Notes

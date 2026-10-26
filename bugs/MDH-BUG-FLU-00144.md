# MDH-BUG-FLU-00144 — Native accepts wrong-type arguments that the interpreter rejects, for about 25 builtins

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T08:13:30Z
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
- **State history:** Open (2026-10-25T08:13:30Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T09:08:04Z, deltic:auto role=fix run=fix-20261025T081928Z-4caa481f branch=task/bug-MDH-BUG-FLU-00144-run-fix-20261025T081928Z-4caa481f code=1a64677 gate=manual) -> Closed (2026-10-26T19:40:44Z, deltic:manual role=verify run=verify-20261026T182021Z-6ed878b5 verified_fix_run=fix-20261025T081928Z-4caa481f verdict=close)

## Observation

Found by lane S10's builtin audit 2026-10-25 (executed): native silently accepts bad argument types where the interpreter raises a type error: atween, hauld_atween, lerp, gcd, lcm, sign, trunc, is_even, is_odd, is_prime, is_positive, is_negative, is_empty, is_toom, enumerate, starts_with, ends_with, trim, trim_start, trim_end, words, scran; tumble accepts the wrong argument order; hauld_atween("hello","l","L") returns a pointer-sized integer; factorial(2.5) hangs and factorial(-3) returns 1; log_enabled/log_init/log_set_filter/log_span/set_log_level/set_crash_handling ignore invalid input; tls_client_new/tls_connect/tls_close return error dicts where the interpreter raises; scrieve returns aye/nae where the interpreter returns naething.

Evidence fingerprint: `manual:v1:native-accepts-wrong-type-arguments-that-the-in-2b6c6cbf3864729c`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T19:40:44Z, deltic:manual run=verify-20261026T182021Z-6ed878b5 verified_fix_run=fix-20261025T081928Z-4caa481f verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] fix=1a64677; corrected CLI repro: hauld_atween returns l in interpreter/native; factorial and starts_with type errors match; 9 relevant tests selected and passed 1/1 each; mutant bypassing factorial negative guard failed numeric assertion: got 1 instead of negative-factorial error.

## Notes

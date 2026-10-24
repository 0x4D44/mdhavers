# MDH-BUG-FLU-00129 — benchmarks/large_scale: tick() deltas mislabelled as ms, and fib_large overflows natively

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** benchmarks
- **Raised:** 2026-10-24T22:25:42Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T224248Z-86473bee
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00129-run-fix-20261024T224248Z-86473bee
- **Owner base:** 07b1df1114fdd53d7849ee49f27a21201972a434
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T22:42:48Z
- **Owner until:** 2026-10-25T02:42:48Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T22:25:42Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane U1 2026-10-24, units checked by the lead: tick() returns nanoseconds in the interpreter (src/interpreter.rs ~8586) and native (inline_tick, CLOCK_REALTIME), but benchmarks/large_scale/primes_large.braw (line 98 etc.) and sort_stress print tick deltas / 1000 labelled "ms" (that is microseconds), e.g. "sieve(1M) ... in 14129 ms" in a 27 ms process. Separately benchmarks/large_scale/fib_large.braw stops natively with "Yer number's too muckle at line 14 - it's overflowed!" (checked i64 overflow, same before and after U1), so the benchmark is unusable natively. Fix: divide by 1000000 (or use mono_ms) and bound fib_large below the i64 limit.

Evidence fingerprint: `manual:v1:benchmarks-large-scale-tick-deltas-mislabelled--73ee84f26b7224a6`


## Fix

<unfixed — raised only>

## Notes

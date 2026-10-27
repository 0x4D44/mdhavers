# MDH-BUG-FLU-00129 — benchmarks/large_scale: tick() deltas mislabelled as ms, and fib_large overflows natively

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** benchmarks
- **Raised:** 2026-10-24T22:25:42Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261027T045159Z-f9fe2891
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00129-run-verify-20261027T045159Z-f9fe2891
- **Owner base:** b2712adc7b304eb842407eae833911cd438de665
- **Owner fingerprint:** sha256:0ace0aa28fc4bb1a2da717d2479d3cb341077d66c4823c98d67940f61dac0d77
- **Owner since:** 2026-10-27T04:51:59Z
- **Owner until:** 2026-10-27T08:51:59Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=1, indeterminate=0
- **State history:** Open (2026-10-24T22:25:42Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T00:08:10Z, deltic:auto role=fix run=fix-20261024T224248Z-86473bee branch=task/bug-MDH-BUG-FLU-00129-run-fix-20261024T224248Z-86473bee code=7af9b14 gate=manual) -> Open (2026-10-26T23:48:13Z, deltic:manual role=verify run=verify-20261026T224201Z-9f60c95c verified_fix_run=fix-20261024T224248Z-86473bee verdict=doubt reason=reopen-regression-assertion-exists-to-run-or-mutate) -> Fixed (2026-10-27T04:50:59Z, deltic:auto role=fix run=fix-20261027T015311Z-86738477 branch=task/bug-MDH-BUG-FLU-00129-run-fix-20261027T015311Z-86738477 code=4c29d34 gate=manual)

## Observation

Found by lane U1 2026-10-24, units checked by the lead: tick() returns nanoseconds in the interpreter (src/interpreter.rs ~8586) and native (inline_tick, CLOCK_REALTIME), but benchmarks/large_scale/primes_large.braw (line 98 etc.) and sort_stress print tick deltas / 1000 labelled "ms" (that is microseconds), e.g. "sieve(1M) ... in 14129 ms" in a 27 ms process. Separately benchmarks/large_scale/fib_large.braw stops natively with "Yer number's too muckle at line 14 - it's overflowed!" (checked i64 overflow, same before and after U1), so the benchmark is unusable natively. Fix: divide by 1000000 (or use mono_ms) and bound fib_large below the i64 limit.

Evidence fingerprint: `manual:v1:benchmarks-large-scale-tick-deltas-mislabelled--73ee84f26b7224a6`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:48:13Z, deltic:manual run=verify-20261026T224201Z-9f60c95c verified_fix_run=fix-20261024T224248Z-86473bee verdict=doubt)

Verifier note: Manual verify: [la luna] 7af9b14 changed 13 benchmark files but added no tests; rg found no benchmark references under tests/. Interpreter primes_large exits 0 (1M: 66,629 ms); native fib_large exits 0 with capped values. No regression assertion exists to run or mutate.

## Notes

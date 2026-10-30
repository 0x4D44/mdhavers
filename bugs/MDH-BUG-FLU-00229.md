# MDH-BUG-FLU-00229 — stdlib random helpers use jammy(0, len-1) and never pick the last item

- **State:** Open
- **Priority:** Could
- **Severity:** Medium
- **Area:** stdlib
- **Raised:** 2026-10-30T07:14:12Z
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
- **State history:** Open (2026-10-30T07:14:12Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

`jammy(a, b)` is half-open (a..b-1, src/interpreter.rs:6374-6380). Over 2000-3000 trials: proptesting gen_bool never true, gen_digit/gen_char never `9`, gen_one_of never the last element and hurls on a one-element list; scots random_* never the last GREETINGS/PROVERBS; crypto generate_simple_key never `9`; test.braw TestData.random_bool always `nae`, random_string never `z`. Sites: proptesting.braw:34,40,47,54,61,110; scots.braw:174,180,186,192,198,294,308; crypto.braw:408; test.braw:451,466.

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:stdlib-random-helpers-use-jammy-0-len-1-and-nev-d13b033474fc0786`


## Fix

<unfixed — raised only>

## Notes

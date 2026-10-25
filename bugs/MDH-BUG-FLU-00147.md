# MDH-BUG-FLU-00147 — CLI interpreter and library interpreter disagree on prelude-overridden builtins

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-25T08:13:42Z
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
- **State history:** Open (2026-10-25T08:13:42Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane S10's builtin audit 2026-10-25 (executed): mdhavers run loads stdlib/prelude.braw, whose definitions replace the Rust natives for average, median, flatten, unique, the_noo, ony and backside_forrit; the library interpreter used by the goldens does not. E.g. average([]) and median([]) error in the library interpreter (golden tests/golden/edge_cases/average_median_parity_and_errors.braw pins that) but give 0 and naething under the CLI; backside_forrit("hello") gives a char list under the CLI and "olleh" natively (pinned by the bug-140 test). Same class as MDH-BUG-FLU-00115 (where Arthur chose native wins); needs the same decision for these names.

Evidence fingerprint: `manual:v1:cli-interpreter-and-library-interpreter-disagre-c327070269187f7e`


## Fix

<unfixed — raised only>

## Notes

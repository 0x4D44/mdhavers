# MDH-BUG-FLU-00197 — eof_helpers test fails on trunk: word_scramble prints nothing after the end-of-input message

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** examples
- **Raised:** 2026-10-26T23:46:31Z
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
- **State history:** Open (2026-10-26T23:46:31Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

On trunk 5122a4d, `cargo test --test eof_helpers` fails `examples_end_cleanly_at_end_of_input_in_the_interpreter`:
"examples/games/word_scramble.braw: nothing printed after the input ended".

c8b69c4 (MDH-BUG-FLU-00191) added the assertion that each example prints something after the
end-of-input message. word_scramble's end-of-input path sets quit_requested, and the closing branch
then prints only a blank line, so the assertion fails. Deterministic (no randomness on this path).
The S28 lead check did not run the eof_helpers target. Found by the LA Luna verify batch (00156/00175 notes).

Evidence fingerprint: `trunk-red:v2:mdhavers-eof-helpers-test-fails-on-trunk-word-s-ee0cb514ce7ef899`


## Fix

<unfixed — raised only>

## Notes

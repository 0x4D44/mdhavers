# MDH-BUG-FLU-00116 — JS upper, heid, tail, bum, reverse and split("") differ from the interpreter on Unicode

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-24T03:17:19Z
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
- **State history:** Open (2026-10-24T03:17:19Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane J6 2026-10-24 via js_golden_parity reasons: upper("ß") gives STRASSE in JS but keeps ß in the interpreter; heid/tail/bum/reverse split astral characters (emoji) by UTF-16 unit; split("") differs. Remaining allowlist entries in tests/js_parity_allowlist.txt name these.

Evidence fingerprint: `manual:v1:js-upper-heid-tail-bum-reverse-and-split-differ-7d26aa57ccd6f064`


## Fix

<unfixed — raised only>

## Notes

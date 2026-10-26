# MDH-BUG-FLU-00116 — JS upper, heid, tail, bum, reverse and split("") differ from the interpreter on Unicode

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-24T03:17:19Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T225713Z-e115a604
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00116-run-verify-20261026T225713Z-e115a604
- **Owner base:** 6cca1b9724e089b14af0018739faf304b5548a5a
- **Owner fingerprint:** sha256:452a638b40ab7226f5b9ce3c7e05d9d988e54476207bd0216ec282e01a3ecc36
- **Owner since:** 2026-10-26T22:57:13Z
- **Owner until:** 2026-10-27T02:57:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T03:17:19Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T03:21:58Z, deltic:auto role=fix run=fix-20261024T032152Z-3f2b6187 branch=task/bug-MDH-BUG-FLU-00116-run-fix-20261024T032152Z-3f2b6187 code=ec8acdf gate=manual)

## Observation

Found by lane J6 2026-10-24 via js_golden_parity reasons: upper("ß") gives STRASSE in JS but keeps ß in the interpreter; heid/tail/bum/reverse split astral characters (emoji) by UTF-16 unit; split("") differs. Remaining allowlist entries in tests/js_parity_allowlist.txt name these.

Evidence fingerprint: `manual:v1:js-upper-heid-tail-bum-reverse-and-split-differ-7d26aa57ccd6f064`


## Fix

<unfixed — raised only>

## Notes

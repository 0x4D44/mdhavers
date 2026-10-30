# MDH-BUG-FLU-00222 — chynge/dicht: index below -len wraps to a huge unsigned index

- **State:** Open
- **Priority:** Could
- **Severity:** Low
- **Area:** interpreter
- **Raised:** 2026-10-30T07:06:28Z
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
- **State history:** Open (2026-10-30T07:06:28Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

**Symptom:** a negative index below minus the list length wraps to a huge unsigned value instead of a sensible bounds error.

**Repro (interpreter, 0.3.137, observed):** `blether chynge([1,2], -5, 0)` prints
`Jings! Something went awfy wrang: Index 18446744073709551613 oot o' bounds fer list o' length 2`.
Expected: an out-of-bounds error naming index -5.

**Where:** src/interpreter.rs:~6834 (`chynge`); `dicht` at ~6856 looks the same (not run).
Found by the docs review, 2026-10-30; reproduced by the lead.

Evidence fingerprint: `manual:v1:chynge-dicht-index-below-len-wraps-to-a-huge-unsigned-index`


## Fix

<unfixed — raised only>

## Notes

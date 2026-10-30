# MDH-BUG-FLU-00248 — maths range_step with a zero step loops forever and exhausts memory

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** stdlib
- **Raised:** 2026-10-30T07:21:56Z
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
- **State history:** Open (2026-10-30T07:21:56Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

**Symptom:** `range_step(5, 0, 0)` (zero step with start > finish) never returns and grows memory without bound. An orphaned repro reached 41 GB RSS in 12 minutes on FLUX, 2026-10-30, and forced a site build hold.

**Repro (interpreter, 0.3.137):** `fetch "stdlib/maths"` then `range_step(5, 0, 0)`. Run only under `deltic timeout 5` and a memory cap. Expected: a hurl for a zero step (or `[]`). `range_step(0, 5, 0)` returns `[]`.

**Where:** stdlib/maths.braw `range_step`: a non-positive step takes the counting-down branch while `i > finish`, and a zero step never changes `i`, so the list grows forever.

Evidence fingerprint: `manual:v1:maths-range-step-with-a-zero-step-loops-forever-0f0f6b879059e176`


## Fix

<unfixed — raised only>

## Notes

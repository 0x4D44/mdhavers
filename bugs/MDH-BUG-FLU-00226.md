# MDH-BUG-FLU-00226 — json tae_json/pretty_json/json_type misclassify numbers, booleans and null

- **State:** Open
- **Priority:** Could
- **Severity:** Medium
- **Area:** stdlib
- **Raised:** 2026-10-30T07:13:53Z
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
- **State history:** Open (2026-10-30T07:13:53Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

`tae_json(42)` gives `"42"`, `tae_json(aye)` gives `"aye"`, `tae_json(naething)` gives `"naething"`, nested numbers are quoted; `json_type(42)`/`(aye)`/`(naething)` give `unknown`. Expected `42`, `true`, `null`, and `number`/`boolean`/`null`. stdlib/json.braw:13,14,20 and :442-444 switch on `"nil"/"boolean"/"number"` but `whit_kind` returns `naething`/`bool`/`integer`/`float`.

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:json-tae-json-pretty-json-json-type-misclassify-76c96770e06f4862`


## Fix

<unfixed — raised only>

## Notes

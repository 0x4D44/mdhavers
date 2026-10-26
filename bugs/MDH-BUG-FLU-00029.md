# MDH-BUG-FLU-00029 — mdhavers fmt does not re-escape string literals

- **State:** Closed
- **Priority:** Should
- **Severity:** High
- **Area:** formatter
- **Raised:** 2026-10-22T23:24:09Z
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
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:58:22Z, deltic:auto role=fix run=fix-20261023T015816Z-fa4529d4 branch=task/bug-MDH-BUG-FLU-00029-run-fix-20261023T015816Z-fa4529d4 code=6e06c31 gate=manual) -> Closed (2026-10-26T22:40:14Z, deltic:manual role=verify run=verify-20261026T214519Z-184b001a verified_fix_run=fix-20261023T015816Z-fa4529d4 verdict=close)

## Observation

Confirmed: "say \"hi\"" is written as "say "hi"" (parse error); "back\\nslash" becomes a real newline (meaning change); single-quoted strings containing " break. ast.rs:411 Literal Display.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T22:40:14Z, deltic:manual run=verify-20261026T214519Z-184b001a verified_fix_run=fix-20261023T015816Z-fa4529d4 verdict=close)

Verifier note: Manual verify: [o1b sol] 6e06c31: fmt_strings_are_re_escaped selected/pass; string escaper bypassed; expected-success assertion rejects unescaped strings (lexer error). CLI keeps escaped quotes/backslash and --check passes; restored clean.

## Notes

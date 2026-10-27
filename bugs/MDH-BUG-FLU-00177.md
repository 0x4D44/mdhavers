# MDH-BUG-FLU-00177 — haud_yer_wheesht() cannot be called: the lexer reads haud as a keyword and the parse fails

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** parser
- **Raised:** 2026-10-26T05:32:14Z
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
- **State history:** Open (2026-10-26T05:32:14Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T22:26:10Z, deltic:auto role=fix run=fix-20261026T222505Z-720f6381 branch=task/bug-MDH-BUG-FLU-00177-run-fix-20261026T222505Z-720f6381 code=e8e6bda gate=manual) -> Closed (2026-10-27T02:37:15Z, deltic:manual role=verify run=verify-20261027T013251Z-669af31a verified_fix_run=fix-20261026T222505Z-720f6381 verdict=close)

## Observation

Found by lane S23 2026-10-26 (executed): haud_yer_wheesht() is an interpreter builtin (and listed in docs/builtins), but a program calling it fails to parse on every backend: the lexer tokenises the haud prefix as the keyword haud. Check src/lexer.rs / src/token.rs keyword matching (keywords should only match whole identifiers; logos priority or a missing word boundary). Also check other builtin names that start with a keyword (log_* were noted as parser keywords by S16). Fix and add a parse test per affected name.

Evidence fingerprint: `manual:v1:haud-yer-wheesht-cannot-be-called-the-lexer-rea-13fd7007e918c17d`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T02:37:15Z, deltic:manual run=verify-20261027T013251Z-669af31a verified_fix_run=fix-20261026T222505Z-720f6381 verdict=close)

Verifier note: Manual verify: Independent Opus review (lead landed this, so lead did not decide): close - round-4 decision keeps haud_yer_wheesht as a continue keyword and removes the dead builtin, so haud_yer_wheesht() parsing as the keyword is decided behaviour; builtin gone from interpreter/LLVM; covered by lexer test_keywords_only_match_whole_identifiers and interpreter haud_yer_wheesht_is_a_keyword_not_a_builtin; builtin-arm mutant fails the coverage test's assertion. Verifier: [le luna] fix e8e6bda; cfg(coverage) test passed and failed its expect_err assertion when the LLVM builtin arm was restored, but the original haud_yer_wheesht() repro still exits 1: “Unexpected (” because the lexer treats it as the haud keyword.

## Notes

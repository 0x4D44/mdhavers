# MDH-BUG-FLU-00177 — haud_yer_wheesht() cannot be called: the lexer reads haud as a keyword and the parse fails

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** parser
- **Raised:** 2026-10-26T05:32:14Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261026T153300Z-435612fd
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00177-run-fix-20261026T153300Z-435612fd
- **Owner base:** 4ab0a76055547c9686ddcd004755e5b3afd1e1c0
- **Owner fingerprint:** -
- **Owner since:** 2026-10-26T15:33:00Z
- **Owner until:** 2026-10-26T19:33:00Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T05:32:14Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane S23 2026-10-26 (executed): haud_yer_wheesht() is an interpreter builtin (and listed in docs/builtins), but a program calling it fails to parse on every backend: the lexer tokenises the haud prefix as the keyword haud. Check src/lexer.rs / src/token.rs keyword matching (keywords should only match whole identifiers; logos priority or a missing word boundary). Also check other builtin names that start with a keyword (log_* were noted as parser keywords by S16). Fix and add a parse test per affected name.

Evidence fingerprint: `manual:v1:haud-yer-wheesht-cannot-be-called-the-lexer-rea-13fd7007e918c17d`


## Fix

<unfixed — raised only>

## Notes

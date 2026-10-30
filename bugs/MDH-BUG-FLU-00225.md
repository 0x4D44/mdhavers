# MDH-BUG-FLU-00225 — rtcp_parse_rr hangs on a truncated packet (haud skips the index increment)

- **State:** Open
- **Priority:** Could
- **Severity:** Medium
- **Area:** stdlib
- **Raised:** 2026-10-30T07:13:46Z
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
- **State history:** Open (2026-10-30T07:13:46Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

`ken b = rtcp_rr(1, []); bytes_set(b, 0, 130); blether rtcp_parse_rr(b)` never returns (killed by a 10 s timeout, exit 124). Expected an `{"ok": nae, ...}` dict. stdlib/rtcp.braw:83: `haud` (continue) skips `i = i + 1`.

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:rtcp-parse-rr-hangs-on-a-truncated-packet-haud--05bc6dcaddc6c7ea`


## Fix

<unfixed — raised only>

## Notes

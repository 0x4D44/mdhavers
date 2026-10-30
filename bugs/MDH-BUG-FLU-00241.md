# MDH-BUG-FLU-00241 — tls tls_client/tls_server mutate the caller's config dict

- **State:** Open
- **Priority:** Could
- **Severity:** Low
- **Area:** stdlib
- **Raised:** 2026-10-30T07:15:26Z
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
- **State history:** Open (2026-10-30T07:15:26Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

`ken mine = {"server_name": "x"}; tls_client(mine)` leaves `mine` with `"mode": client`; `tls_server(mine)` sets `"mode": server` even when it then hurls. tls.braw:7-8,15-16.

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:tls-tls-client-tls-server-mutate-the-caller-s-config-dict`


## Fix

<unfixed — raised only>

## Notes

# MDH-BUG-FLU-00131 — Native dtls_handshake: probable double close of the dup fd on a failed handshake

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-25T01:32:22Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261025T020015Z-990f2c0f
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00131-run-fix-20261025T020015Z-990f2c0f
- **Owner base:** eb244602852300eaa2d7efc56aa6732035949537
- **Owner fingerprint:** -
- **Owner since:** 2026-10-25T02:00:15Z
- **Owner until:** 2026-10-25T06:00:15Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T01:32:22Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane R0a 2026-10-25 (read, not executed), checked by the lead reading the code: __mdh_dtls_handshake (runtime/mdh_runtime.c ~3211) dups the socket fd and passes it to __mdh_rs_dtls_handshake (runtime/mdh_runtime_rs/src/lib.rs ~1617), which takes ownership (the socket it wraps closes the fd when dropped on its error paths). The C error branch then calls mdh_socket_close(dup_fd) again, which can close an unrelated fd that reused the number. Confirm whether the Rust side closes on every error path; if so, drop the C close (or have Rust return ownership).

Evidence fingerprint: `manual:v1:native-dtls-handshake-probable-double-close-of--8215209252ac91b4`


## Fix

<unfixed — raised only>

## Notes

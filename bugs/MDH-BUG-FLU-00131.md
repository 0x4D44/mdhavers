# MDH-BUG-FLU-00131 — Native dtls_handshake: probable double close of the dup fd on a failed handshake

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-25T01:32:22Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T182021Z-640b9af0
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00131-run-verify-20261026T182021Z-640b9af0
- **Owner base:** 2f352f49ef4c71b45a0497bf81a04779c622faad
- **Owner fingerprint:** sha256:c120af03273bacc58519cc39b98eca1186572cdd8c67f04864636b16d369ba1d
- **Owner since:** 2026-10-26T18:20:21Z
- **Owner until:** 2026-10-26T22:20:21Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T01:32:22Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T02:15:59Z, deltic:auto role=fix run=fix-20261025T020015Z-990f2c0f branch=task/bug-MDH-BUG-FLU-00131-run-fix-20261025T020015Z-990f2c0f code=6a72241 gate=manual)

## Observation

Found by lane R0a 2026-10-25 (read, not executed), checked by the lead reading the code: __mdh_dtls_handshake (runtime/mdh_runtime.c ~3211) dups the socket fd and passes it to __mdh_rs_dtls_handshake (runtime/mdh_runtime_rs/src/lib.rs ~1617), which takes ownership (the socket it wraps closes the fd when dropped on its error paths). The C error branch then calls mdh_socket_close(dup_fd) again, which can close an unrelated fd that reused the number. Confirm whether the Rust side closes on every error path; if so, drop the C close (or have Rust return ownership).

Evidence fingerprint: `manual:v1:native-dtls-handshake-probable-double-close-of--8215209252ac91b4`


## Fix

<unfixed — raised only>

## Notes

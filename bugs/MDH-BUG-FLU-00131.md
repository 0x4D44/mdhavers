# MDH-BUG-FLU-00131 — Native dtls_handshake: probable double close of the dup fd on a failed handshake

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-25T01:32:22Z
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
- **Attempts:** fix=0, doubt=1, indeterminate=0
- **State history:** Open (2026-10-25T01:32:22Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T02:15:59Z, deltic:auto role=fix run=fix-20261025T020015Z-990f2c0f branch=task/bug-MDH-BUG-FLU-00131-run-fix-20261025T020015Z-990f2c0f code=6a72241 gate=manual) -> Open (2026-10-26T19:40:44Z, deltic:manual role=verify run=verify-20261026T182021Z-640b9af0 verified_fix_run=fix-20261025T020015Z-990f2c0f verdict=doubt reason=reopen-verifier-gpt-6-luna-max-codex-recorded-by-lead-fix6a72241-native-invalid) -> Fixed (2026-10-27T00:55:56Z, deltic:auto role=fix run=fix-20261026T214549Z-c450ac05 branch=task/bug-MDH-BUG-FLU-00131-run-fix-20261026T214549Z-c450ac05 code=990108f gate=manual)

## Observation

Found by lane R0a 2026-10-25 (read, not executed), checked by the lead reading the code: __mdh_dtls_handshake (runtime/mdh_runtime.c ~3211) dups the socket fd and passes it to __mdh_rs_dtls_handshake (runtime/mdh_runtime_rs/src/lib.rs ~1617), which takes ownership (the socket it wraps closes the fd when dropped on its error paths). The C error branch then calls mdh_socket_close(dup_fd) again, which can close an unrelated fd that reused the number. Confirm whether the Rust side closes on every error path; if so, drop the C close (or have Rust return ownership).

Evidence fingerprint: `manual:v1:native-dtls-handshake-probable-double-close-of--8215209252ac91b4`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T19:40:44Z, deltic:manual run=verify-20261026T182021Z-640b9af0 verified_fix_run=fix-20261025T020015Z-990f2c0f verdict=doubt)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] fix=6a72241; native invalid-handle repro returned Unknown DTLS handle and original socket_close=aye; no DTLS test exists under tests/llvm_*.rs (fix commit adds only 00132 chunks test), so no selected assertion can be mutation-checked.

## Notes

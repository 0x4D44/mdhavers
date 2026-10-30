# srtp

Wrappers over the native SRTP built-ins for protecting and unprotecting RTP packets (see [rtp](./rtp.md)). Build a context from a key dict or from DTLS-derived keys.

Import: `fetch "stdlib/srtp"`

Native only: needs the `srtp_create`, `srtp_protect` and `srtp_unprotect` built-ins, which come from the optional `libsrtp` feature. In the release binary used to check these docs they were not defined (`'srtp_create' hasnae been defined yet`), so build with the feature before relying on this module. Results are dicts `{"ok": aye, "value": ...}`; the context is the integer `value`. This module hurls nothing itself.

## Functions

### srtp_new(keys)
Calls `srtp_create(keys)` with the dict as given. Keys: `profile` (default `"SRTP_AES128_CM_SHA1_80"`; unknown gives `ok: nae`, "Unsupported SRTP profile"), `role`, and keys as bytes: either `send_key`/`send_salt`/`recv_key`/`recv_salt`, or `client_key`/`client_salt`/`server_key`/`server_salt`, or `master_key`/`master_salt` for both directions.

### srtp_from_dtls(keys, role = "client")
Builds a config dict from a DTLS keying result and calls `srtp_create`. `keys` must contain `profile`, `client_key`, `client_salt`, `server_key` and `server_salt`; `role` is `"client"` or `"server"`.

### srtp_protect_o(ctx, packet)
Encrypts and authenticates an RTP packet (`srtp_protect`).

### srtp_unprotect_o(ctx, packet)
Verifies and decrypts an SRTP packet (`srtp_unprotect`).

## Example

_Checked, not run: needs a live peer/device (and the `libsrtp` feature)._

```mdhavers
fetch "stdlib/srtp"
fetch "stdlib/rtp"

ken cfg = {"profile": "SRTP_AES128_CM_SHA1_80", "master_key": bytes(16), "master_salt": bytes(14)}
ken ctx = srtp_new(cfg)["value"]
ken pkt = rtp_packet("payload", 1, 160, 1234)
ken protected = srtp_protect_o(ctx, pkt)["value"]
blether srtp_unprotect_o(ctx, protected)["ok"]
```

# tls

Wrappers over the native TLS built-ins, for keeping SIP (or other TCP) traffic encrypted. Create a client or server session from a config dict, handshake over a socket from [network_real](./network_real.md), then send and receive bytes.

Import: `fetch "stdlib/tls"`

Native only: needs the TLS built-ins (`tls_client_new`, `tls_connect`, `tls_send`, `tls_recv`, `tls_close`) and `bytes_from_string`. Not available on the JavaScript or WASM backends. This module hurls no errors of its own.

## Functions

### tls_client(config = {})
Copies `config` (note: the copy is shallow-by-reference, so the caller's dict also gains `"mode"`), sets `"mode"` to `"client"`, and calls `tls_client_new`. Returns a result dict `{"ok": aye, "value": <session id>}`. Recognised config keys include `server_name` (default `"localhost"`), `insecure` (bool), `ca_pem`, `cert_pem`, `key_pem`. Pass the session id (`value`) to the other functions.

### tls_server(config = {})
As `tls_client`, but sets `"mode"` to `"server"` (it also calls `tls_client_new`). A server needs `cert_pem` and `key_pem` in the config; without `cert_pem` the built-in hurls `Server cert_pem is required`.

### tls_connect_o(tls, sock)
Performs the handshake over `sock` (`tls_connect`).

### tls_send_o(tls, buf)
Sends a bytes buffer (`tls_send`).

### tls_send_string(tls, text)
Converts `text` with `bytes_from_string` and sends it.

### tls_recv_o(tls, max_len)
Receives up to `max_len` bytes (`tls_recv`).

### tls_close_o(tls)
Closes the session (`tls_close`).

## Example

_Checked, not run: needs a live peer/device._

```mdhavers
fetch "stdlib/tls"
fetch "stdlib/network_real"

ken sock = tcp_socket()
connect_socket(sock["value"], "example.com", 443)
ken session = tls_client({"server_name": "example.com"})["value"]
tls_connect_o(session, sock["value"])
tls_send_string(session, "hello")
tls_close_o(session)
```

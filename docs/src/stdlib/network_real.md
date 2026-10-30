# network_real

Thin, named wrappers over the native socket built-ins (`socket_udp`, `socket_tcp`, `socket_bind`, ...): create UDP/TCP sockets, bind, connect, listen, accept, tune options, send and receive bytes, and look up DNS. Each wrapper simply forwards its arguments and returns whatever the built-in returns: a result dict `{"ok": aye, "value": ...}` (on failure `ok` is `nae` with an error). A socket is an integer id held in `value`; pass that id (not the dict) to the other functions. Passing anything else is hurled by the built-in, for example `socket_bind() expects socket id`.

Import: `fetch "stdlib/network_real"`

Native only: needs the interpreter/native build with the socket built-ins. It does not apply to the JavaScript or WASM backends. Helpers starting with `_` are internal and not documented (this module has none). Errors come from the underlying built-ins; this module hurls nothing itself.

## Functions

### udp_socket()
New UDP socket (`socket_udp()`).

### tcp_socket()
New TCP socket (`socket_tcp()`).

### bind_socket(sock, host = naething, port = 0)
Binds `sock` (`socket_bind`). Port 0 lets the OS choose.

### connect_socket(sock, host, port)
Connects `sock` (`socket_connect`).

### listen_socket(sock, backlog = 128)
Starts listening (`socket_listen`).

### accept_socket(sock)
Accepts a connection (`socket_accept`).

### set_nonblocking(sock, on = aye)
Toggles non-blocking mode.

### set_reuseaddr(sock, on = aye)
Toggles `SO_REUSEADDR`.

### set_reuseport(sock, on = aye)
Toggles `SO_REUSEPORT`.

### set_ttl(sock, ttl)
Sets the IP TTL.

### set_nodelay(sock, on = aye)
Toggles TCP_NODELAY.

### set_rcvbuf(sock, bytes)
Sets the receive buffer size.

### set_sndbuf(sock, bytes)
Sets the send buffer size.

### close_socket(sock)
Closes the socket (`socket_close`).

### udp_send(sock, buf, host, port)
Sends a bytes buffer to `host:port` (`udp_send_to`).

### udp_recv(sock, max_len)
Receives up to `max_len` bytes (`udp_recv_from`).

### tcp_send_bytes(sock, buf)
Sends a bytes buffer on a connected socket (`tcp_send`).

### tcp_recv_bytes(sock, max_len)
Receives up to `max_len` bytes (`tcp_recv`).

### dns_lookup_real(host)
Calls the `dns_lookup` built-in. In [sip](./sip.md) the result is used as a dict with `"ok"` and `"value"` (list of addresses).

## Example

_Checked, not run: needs a live peer/device._

```mdhavers
fetch "stdlib/network_real"

ken s = udp_socket()["value"]
bind_socket(s, "127.0.0.1", 0)
udp_send(s, bytes_from_string("ping"), "127.0.0.1", 9000)
close_socket(s)
```

## Compared with network

[network](./network.md) is an in-process simulation (channels, topics, RPC) with no I/O. `network_real` does real I/O through OS sockets and returns raw built-in results. Prefer `network` for tests and in-memory messaging, `network_real` for actual traffic ([sip](./sip.md), [rtp](./rtp.md) and [tls](./tls.md) build on the same wire-level pieces).

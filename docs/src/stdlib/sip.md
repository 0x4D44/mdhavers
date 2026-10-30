# sip

Helpers for SIP: DNS-based server resolution (NAPTR, SRV, then A/AAAA), message parsing and building, and a simple UDP retransmit schedule with transaction dicts. It parses and builds text; it does not open sockets itself (use [network_real](./network_real.md) and [tls](./tls.md)).

Import: `fetch "stdlib/sip"`

Native only for `sip_resolve`, which calls the `dns_naptr`, `dns_srv` and `dns_lookup` built-ins. The rest is pure mdhavers. This module hurls no errors of its own; bad input yields `{"type": "invalid", ...}` or empty lists. Helpers starting with `_` are internal and not documented.

## Constants

`SIP_VERSION` (`"SIP/2.0"`), `SIP_PORT` (5060), `SIPS_PORT` (5061), `SIP_TRANSPORT_UDP` (`"udp"`), `SIP_TRANSPORT_TCP` (`"tcp"`), `SIP_TRANSPORT_TLS` (`"tls"`).

## Functions

### sip_transport_defaults(transport)
Returns `{"transport", "service", "naptr", "port"}` for `"udp"`, `"tcp"` or `"tls"`/`"sips"` (case-insensitive). Anything else falls back to the UDP service, NAPTR and port, but keeps the lower-cased name in `"transport"`.

### sip_trim_dot(name)
Strips one trailing `.` from a DNS name (as a string).

### sip_sort_naptr(records)
Sorts NAPTR record dicts in place by `order` then `preference` (bubble sort); returns the list.

### sip_sort_srv(records)
Sorts SRV dicts in place by `priority` then `weight`; returns the list.

### sip_srv_entry(srv, transport)
Turns an SRV record into `{"target" (trailing dot trimmed), "port", "priority", "weight", "transport"}`.

### sip_resolve(domain, transport = "udp")
Returns a list of server entries (same shape as `sip_srv_entry`). Tries NAPTR (filtered by the transport's service, then SRV on the replacement), then SRV on `_sip._udp` etc., then A/AAAA with the default port. Returns `[]` if nothing resolves.

### sip_bytes_to_string(msg)
Bytes to string (one character per byte); non-bytes go through `tae_string`.

### sip_parse_headers(lines)
List of `Name: value` lines to a dict with lower-case names. Blank lines and lines without `:` are skipped.

### sip_parse_message(msg)
Parses a request or response (string or bytes; `\r\n` or `\n`). Returns one of:

- `{"type": "request", "method", "uri", "version", "headers", "body"}`
- `{"type": "response", "version", "status" (integer), "reason", "headers", "body"}`
- `{"type": "invalid", "raw": text}` when the start line has fewer than two tokens.

### sip_header_get(headers, name, default = naething)
Case-insensitive lookup in a dict produced by `sip_parse_headers`.

### sip_build_message(start_line, headers = {}, body = "")
Joins start line and `Key: value` headers with CRLF and a blank line, then the body. If `body` is non-empty and no `content-length`/`Content-Length` is present, adds `Content-Length` (this mutates the `headers` dict you pass).

### sip_build_request(method, uri, headers = {}, body = "")
Start line `METHOD uri SIP/2.0`.

### sip_build_response(status, reason, headers = {}, body = "")
Start line `SIP/2.0 status reason`.

### sip_build_request_bytes(method, uri, headers = {}, body = "")
As `sip_build_request`, as bytes.

### sip_build_response_bytes(status, reason, headers = {}, body = "")
As `sip_build_response`, as bytes.

### sip_retransmit_delay(attempt, t1 = 500, t2 = 4000)
`t1 * 2^attempt`, capped at `t2` (milliseconds).

### sip_txn_new(request, addr, t1 = 500, t2 = 4000)
New transaction dict: `request`, `addr`, `state` (`"calling"`), `attempt` (0), `t1`, `t2`, `next_delay` (`t1`).

### sip_txn_next(txn)
Increments `attempt`, recomputes `next_delay`; returns the same dict.

### sip_txn_complete(txn)
Sets `state` to `"completed"`; returns the dict.

## Example

```mdhavers
fetch "stdlib/sip"

ken req = sip_build_request("INVITE", "sip:bob@example.com", {"Call-ID": "abc"}, "hello")
ken msg = sip_parse_message(req)
blether msg["method"]
blether sip_header_get(msg["headers"], "Call-ID")
blether sip_retransmit_delay(3)
```

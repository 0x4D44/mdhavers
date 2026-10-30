# rtp

Build and parse RTP packets (RFC 3550 fixed header plus optional CSRC list) as bytes values. Protect packets with [srtp](./srtp.md); pair with [rtcp](./rtcp.md) for reports.

Import: `fetch "stdlib/rtp"`

Pure mdhavers on the bytes built-ins (`bytes`, `bytes_set`, `bytes_write_u16be`, ...); no sockets. This module hurls nothing; short input returns an error dict.

## Functions

### rtp_header(seq, timestamp, ssrc, payload_type = 0, marker = 0, csrcs = [])
Returns a bytes header: version 2, no padding or extension, up to 15 CSRCs (extra ones are ignored). `marker` is taken modulo 2, `payload_type` modulo 128.

### rtp_packet(payload, seq, timestamp, ssrc, payload_type = 0, marker = 0, csrcs = [])
Header plus payload. A non-bytes payload is converted with `tae_string` then `bytes_from_string`.

### rtp_parse(packet)
Parses bytes (or a string, converted). On success returns `{"ok": aye, "version", "padding", "extension", "csrc_count", "marker", "payload_type", "seq", "timestamp", "ssrc", "header_len", "csrcs", "payload"}` (`padding`, `extension` and `marker` are booleans, `payload` is bytes). Failures: `{"ok": nae, "error": "rtp packet too short", "len": n}` (under 12 bytes) or `{"ok": nae, "error": "rtp header too short", "len": n}`. Header extensions and padding are not stripped from the payload.

### rtp_seq_next(seq)
`seq + 1`, wrapping to 0 after 65535.

## Example

```mdhavers
fetch "stdlib/rtp"

ken pkt = rtp_packet("abc", 65535, 160, 99, 8, 1, [7])
ken info = rtp_parse(pkt)
blether info["seq"]
blether info["payload_type"]
blether rtp_seq_next(info["seq"])
```

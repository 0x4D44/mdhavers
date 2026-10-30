# rtcp

Build and parse RTCP Receiver Report (RR, packet type 201) packets as bytes. Pair with [rtp](./rtp.md).

Import: `fetch "stdlib/rtcp"`

Pure mdhavers on the bytes built-ins; no sockets. This module hurls nothing; malformed input returns an error dict.

## Functions

### rtcp_rr(sender_ssrc, reports = [])
Returns bytes for an RR from `sender_ssrc`. Each report is a dict with `ssrc`, `fraction_lost`, `cumulative_lost`, `highest_seq`, `jitter`, `lsr`, `dlsr` (missing keys default to 0). At most 31 reports are written.

### rtcp_parse_rr(packet)
Parses an RR (bytes, or a string that is converted). Success: `{"ok": aye, "version", "padding", "count", "packet_type", "length", "ssrc", "reports"}` where each report has the seven keys above. Failures: `{"ok": nae, "error": "rtcp packet too short", "len": n}`, `{"ok": nae, "error": "not RTCP RR", "packet_type": pt}`, `{"ok": nae, "error": "rtcp length mismatch", "len": n, "expected": m}`.

Caution: a packet whose report-count field claims more reports than its length field allows makes the parser loop forever (see the source `haud` inside the `whiles` loop). Only parse packets you trust or have length-checked.

## Example

```mdhavers
fetch "stdlib/rtcp"

ken pkt = rtcp_rr(1, [{"ssrc": 2, "fraction_lost": 3, "cumulative_lost": 4, "highest_seq": 5, "jitter": 6}])
ken info = rtcp_parse_rr(pkt)
blether info["count"]
blether info["reports"][0]["jitter"]
```

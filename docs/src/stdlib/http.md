# http

HTTP helpers: method and status constants, URL parsing, headers, request and response objects, query-string helpers, cookies and a mock client for tests. There is no real network client here: `Request` only describes a request, and `MockClient` answers from canned responses.

Import: `fetch "stdlib/http"`

Pure mdhavers; works wherever the interpreter does. Loading the module prints `HTTP module loaded! Ready tae fetch frae the web!`. This module hurls no errors of its own. Helpers starting with `_` are internal and not documented.

## Constants

- Methods: `HTTP_GET`, `HTTP_POST`, `HTTP_PUT`, `HTTP_DELETE`, `HTTP_PATCH`, `HTTP_HEAD`, `HTTP_OPTIONS` (the upper-case method names).
- Status codes: `HTTP_OK` 200, `HTTP_CREATED` 201, `HTTP_ACCEPTED` 202, `HTTP_NO_CONTENT` 204, `HTTP_MOVED` 301, `HTTP_FOUND` 302, `HTTP_NOT_MODIFIED` 304, `HTTP_BAD_REQUEST` 400, `HTTP_UNAUTHORIZED` 401, `HTTP_FORBIDDEN` 403, `HTTP_NOT_FOUND` 404, `HTTP_METHOD_NOT_ALLOWED` 405, `HTTP_CONFLICT` 409, `HTTP_GONE` 410, `HTTP_TEAPOT` 418, `HTTP_SERVER_ERROR` 500, `HTTP_NOT_IMPLEMENTED` 501, `HTTP_BAD_GATEWAY` 502, `HTTP_SERVICE_UNAVAILABLE` 503.
- `STATUS_MESSAGES`: dict from the status as a string (`"404"`) to a Scots description.
- `URL_SAFE_CHARS`: letters, digits and `-_.~`.

## Functions

### get_status_message(code)
Scots description for `code`, or `"Unknown status code: <code>"`.

### is_success(code)
`aye` for 200-299.

### is_redirect(code)
`aye` for 300-399.

### is_client_error(code)
`aye` for 400-499.

### is_server_error(code)
`aye` for 500 and up.

### parse_url(url_string)
Returns a `URL`.

### make_headers()
Returns an empty `Headers`.

### get(url)
`Request` with method GET. `url` is a string or a `URL`.

### post(url)
`Request` with method POST.

### put(url)
`Request` with method PUT.

### delete_request(url)
`Request` with method DELETE (named so to avoid clashing with other `delete` names).

### patch(url)
`Request` with method PATCH.

### url_encode(str)
Keeps `URL_SAFE_CHARS`, turns a space into `+`, and leaves every other character unchanged. It does not percent-encode, so `&`, `/` and non-ASCII pass through (source comment: "Would do proper percent encoding here").

### url_decode(str)
Turns `+` into a space only. `%20` is not decoded.

### build_query_string(params)
Joins `key=value` pairs of a dict with `&`, using `url_encode` on each side.

### parse_query_string(query)
Splits on `&` and `=` into a dict of strings, using `url_decode`. Pieces without `=` are dropped.

### make_mock_client()
Returns a `MockClient`.

### make_cookie(name, value)
Returns a `Cookie`.

### make_cookie_jar()
Returns a `CookieJar`.

## Classes

### URL
`URL(url_string)`. Parses into fields `original`, `protocol` (`""` if none), `host`, `port` (integer or `naething`), `path` (default `"/"`), `query` (dict of strings), `fragment`.

- `to_string()`: rebuilds the URL.
- `with_param(key, value)`: sets a query value (as a string); returns the URL.
- `get_param(key)`: the value or `naething`.

### Headers
`Headers()`. Names are case-insensitive (stored lower-case).

- `set(name, value)`, `get(name)` (`naething` if missing), `has(name)`, `remove(name)`.
- `to_dict()`: the lower-cased dict.
- `from_dict(dict)`: sets each pair. `set`, `remove` and `from_dict` return the headers.

### Request
`Request(method, url)`. Fields: `method` (upper-cased), `url` (a `URL`), `headers` (`Headers`), `body` (`naething`), `timeout` (30000 ms).

- `with_header(name, value)`, `with_headers(dict)`, `with_body(body)`, `with_timeout(ms)`: each returns the request.
- `with_json(data)`: sets `content-type: application/json` and the body to `tae_string(data)`. It does not produce real JSON text (source: "Would stringify JSON here").
- `to_dict()`: `method`, `url` (string), `headers`, `body`, `timeout`.

### Response
`Response(status, headers, body)`. A dict for `headers` is converted to `Headers`. Fields: `status`, `headers`, `body`, `ok` (`is_success(status)`).

- `status_text()`: `get_status_message(status)`.
- `json()`: returns the body unchanged (no parsing).
- `text()`: `tae_string(body)`.

### MockClient
`MockClient()`. Fields `responses`, `requests`.

- `mock_response(method, url_pattern, response)`: `response` is a dict with `status`, `headers` and `body`. The key is `METHOD:url_pattern` and must equal the full URL string exactly; there is no pattern matching. Returns the client.
- `send(request)`: records `request.to_dict()` and returns the matching `Response`, or `Response(404, {}, "Not found")`.
- `get_requests()`: recorded request dicts.
- `clear()`: returns the client.

### Cookie
`Cookie(name, value)`. Fields: `name`, `value`, `path` (`"/"`), `domain`, `secure`, `http_only`, `expires` (`naething`, never used by `to_string`).

- `with_path(path)`, `with_domain(domain)`, `secure_only()`, `http_only_flag()`: each returns the cookie.
- `to_string()`: `name=value; Path=/; Domain=...; Secure; HttpOnly`.

### CookieJar
`CookieJar()`.

- `set(cookie)`, `remove(name)`, `clear()`: return the jar.
- `get(name)`: the `Cookie` or `naething`.
- `to_header()`: `a=1; b=2` for a `Cookie` request header.

## Example

```mdhavers
fetch "stdlib/http"

ken mock = make_mock_client()
mock.mock_response("get", "http://h/", {"status": 200, "headers": {"X": "1"}, "body": "ok"})
ken res = mock.send(get("http://h/"))
blether res.status
blether res.text()
blether parse_url("https://ex.com:8080/a?x=1").port
```

# Network HTTP M3 Traceability: URL, Header, And Cookie Primitives

Status: backlog from M0.

| Work item | M0 decision | Acceptance evidence |
| --- | --- | --- |
| `sifr.url.Url` and `UrlQuery` | `production-public`, stable-public-api over ASCII and already-valid Sifr text. | RFC/CPython-derived parse/build/invalid-input fixtures. |
| Percent helpers | Byte/ASCII/UTF-8 safe helpers accepted; named encodings blocked on text/i18n M1. | Percent fixture set and blocked codec diagnostics. |
| Host and authority validation | ASCII domain labels, IPv4, IPv6, and already-punycode accepted; non-ASCII blocked until text/i18n M2; ports are decimal 0-65535. | ASCII and punycode fixtures plus non-ASCII host and invalid-port rejection fixtures. |
| Path normalization | Parsing does not silently normalize; explicit dot-segment helper preserves encoded slash/backslash boundaries. | Security/resource fixtures for path traversal-sensitive cases. |
| Query parsing/building | byte/ASCII/UTF-8 safe behavior only; sensitive query key redaction applies in observability. | Query edge cases and explicit blocked-state records for non-UTF-8 form decoding. |
| `sifr.http` header primitives | M3 owns canonical `HeaderName`, `HeaderValue`, and `HeaderMap`: ASCII token names, obs-fold rejection, OWS trim only, duplicate order preservation, and size caps from inventory. | Header token, obs-fold rejection, duplicate policy, and size-limit fixtures. |
| Cookie header parsing | header-level parse/build only. | Small cookie header fixtures; jars and persistence rejected. |

## CPython Evidence

Mine `urllib.parse`, `http.cookies`, and selected `http.client` header behavior. `urllib.request` and `http.cookiejar` remain rejection/defer evidence.

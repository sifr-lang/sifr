# Text/I18n M1 Traceability

Milestone: `milestone_text_i18n_1`

| Backlog item | Required fixture/evidence |
| --- | --- |
| Static `sifr.encoding.Encoding` registry | Alias-resolution tests for Tier 0/Tier 1 labels; unsupported registry mutation negative fixtures. |
| Encode/decode APIs and typed handlers | `sifr_encoding_subset.sifr`; strict failure, replace/ignore/backslash/name/xmlcharref outcomes. |
| Incremental encoder/decoder | Finalization, exhausted-state, partial input, recoverable diagnostics fixtures. |
| `str.encode` / `bytes.decode` substrate reuse | Method fixtures and emitted-code scan proving one registry. |
| Explicit text I/O | `sifr_text_io_subset.sifr`; `sifr.io.open_text`, `TextReader`, `TextWriter`, `open(..., encoding=...)`. |
| No implicit text encoding | Negative fixtures for `open(path)` / text mode without `encoding=` and dynamic/nonliteral mode. |

Consumer unblock records: network/HTTP text decoding and concurrency/runtime subprocess text mode remain blocked until this milestone closes.

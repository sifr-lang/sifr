# Text/I18n M1 Traceability

Milestone: `milestone_text_i18n_1`

| Backlog item | Required fixture/evidence |
| --- | --- |
| Static `sifr.encoding.Encoding` registry | `crates/sifr/tests/e2e/pass/text_i18n_encoding_io.sifr` covers Tier 0 `ascii`, exact `latin-1`, `utf-8-sig` BOM handling, `utf-16-le`, Tier 1 `windows-1252`, and aliases `cp1252`, `utf_8`, and `iso8859-1`; `verification/stdlib/text_i18n_substrate_inventory.md` remains the exact Tier 0/Tier 1 alias source of truth. |
| Encode/decode APIs and typed handlers | `text_i18n_encoding_io.sifr` covers `decode`, `encode`, `decode_outcome`, `encode_outcome`, strict defaults, `replace` decode recovery, `xmlcharrefreplace` encode recovery diagnostics, method-level `str.encode(..., errors)` / `bytes.decode(..., errors)` propagation, and write-mode text `open(..., errors="xmlcharrefreplace")` encoding. |
| Incremental encoder/decoder | `text_i18n_encoding_io.sifr` covers split UTF-8 decoding across `final=False` then dynamic `final`, `Encoder` dynamic finalization, and post-finalization typed exhausted-state errors. |
| `str.encode` / `bytes.decode` substrate reuse | `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs` asserts method lowering goes through `sifr_runtime::encoding::{encode_bytes,decode_text}` and records `SifrRuntime` plus `EncodingRs` feature metadata; `text_i18n_encoding_io.sifr` covers supported handler propagation. |
| Explicit text I/O | `text_i18n_encoding_io.sifr` covers `sifr.io.open_text(...)` and compiler-special `open(..., encoding=..., errors=...)` over `TextFileHandle`. `TextReader`/`TextWriter` names exist as unsupported direct-construction wrappers; line-buffer cursor semantics are deferred. |
| No implicit text encoding | `crates/sifr/tests/e2e/fail/text_i18n_open_without_encoding.sifr` covers `SIFR-IO-0801`; `text_i18n_open_dynamic_mode.sifr` covers `SIFR-IO-0802`; `text_i18n_dynamic_errors_handler.sifr`, `text_i18n_open_dynamic_errors_handler.sifr`, and `text_i18n_decode_encode_only_handler.sifr` cover `SIFR-ENCODING-0803`; `text_i18n_textiowrapper_unsupported.sifr` and `text_i18n_codecs_register_unsupported.sifr` record Python-shaped unsupported surfaces. |

Consumer unblock records: network/HTTP text decoding and concurrency/runtime subprocess text mode remain blocked until this milestone closes.

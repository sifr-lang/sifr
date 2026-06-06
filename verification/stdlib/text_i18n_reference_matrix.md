# Text/I18n Reference Matrix

CPython checkout: `/Users/yaseralnajjar/work/sifr/cpython` at `14cbd0e6afa98355bdc6749b8230fed4c9b21bd6`.

| Reference | Extracted signal | Sifr disposition |
| --- | --- | --- |
| `Lib/codecs.py` | 12 public functions, 10 public classes, 42 public methods, 11 BOM constants | Static registry, encode/decode, incremental codecs, BOM fixtures mined for M1; stream wrappers and registry mutation deferred or unsupported. |
| `Lib/encodings/*.py` | 121 encoding modules and 343 aliases | Tier 0/Tier 1 labels mined; public `encodings.*` parity rejected/deferred. |
| `Lib/test/test_codecs.py` | 42 classes, 206 test methods | UTF-8/UTF-8-SIG/ASCII/Latin-1/UTF-16 fixtures mined; UTF-32/CJK/text-to-text/pseudo-codec families deferred/rejected. |
| `Lib/test/test_capi/test_codecs.py` | C API codec behavior | `rejected`; external CPython C API compatibility is outside Sifr product scope. |
| `Doc/library/codecs.rst` | Registry/stream/error-handler documentation | Mined for diagnostic wording and unsupported mutation evidence. |
| `Modules/_codecsmodule.c`, `Modules/cjkcodecs/*` | Native codec implementation details | Evidence only; no from-scratch CJK codec work in this phase. |
| `Doc/library/unicodedata.rst` | Unicode property API names | M2 API names and typed error/option behavior adapted for `sifr.unicode`. |
| `Lib/test/test_unicodedata.py` | 7 classes, 42 test methods | Normalization, properties, names/lookup, numeric values, and grapheme tests mined for M2/M2.5. |
| `Modules/unicodedata.c` | CPython table behavior | Evidence for table shape; generated Sifr tables remain Rust-owned and reproducible. |
| `Lib/locale.py` | 12 public functions | `setlocale`, `localeconv`, `strcoll`, `strxfrm`, implicit encodings unsupported; locale ID and formatting behavior adapted to object-scoped M3 APIs. |
| `Lib/test/test_locale.py` | 26 classes, 98 test names | Locale normalization and formatting fixture ideas mined; host real-locale tests marked `external-signal` or `host-limited`. |
| `Doc/library/locale.rst`, `Modules/_localemodule.c` | Process-global locale behavior | Anti-requirement evidence; no process-global mutation. |
| `Lib/gettext.py` | 14 public functions, 2 classes, 12 methods | Fallback and plural behavior adapted to `Bundle`/`Translator`; global install/domain functions unsupported. |
| `Lib/test/test_gettext.py` | 20 classes, 67 test methods, `.mo` binary fixtures | `.mo` parser, plural expressions, contexts, fallbacks, malformed catalog fixtures mined for M4. |
| `Doc/library/gettext.rst` | Domain/global API docs | Backend scope evidence; global `_` mutation unsupported. |

## Selected CPython Alias Evidence

Accepted or mined aliases include canonical labels and WHATWG-compatible names for `utf-8`, `utf-8-sig`, `ascii`, `latin-1`, `utf-16-le`, `utf-16-be`, and Windows-125x labels covered by `encoding_rs`. CPython-only aliases such as broad codepage, CJK, text-to-text, and bytes-to-bytes module families are not accepted in this phase.

## Fixture Plan

| Planned fixture | Owner milestone | Source evidence |
| --- | --- | --- |
| `sifr_encoding_subset.sifr` | M1 | `test_codecs` UTF-8/ASCII/Latin-1/UTF-16/BOM/error-handler tests |
| `sifr_text_io_subset.sifr` | M1 | `codecs.open` and `io` text boundary anti-requirements adapted to `sifr.io.open_text` |
| `sifr_unicode_subset.sifr` | M2 | `test_unicodedata` property/name/normalization tests |
| `sifr_unicode_segmentation_subset.sifr` | M2.5 | `GraphemeBreakTest` and Unicode UAX #29 examples |
| `sifr_i18n_locale_formatting_subset.sifr` | M3 | `test_locale` deterministic formatting/collation families |
| `sifr_i18n_translation_subset.sifr` | M4 | `test_gettext` `.mo`, plural, context, fallback, malformed catalog families |

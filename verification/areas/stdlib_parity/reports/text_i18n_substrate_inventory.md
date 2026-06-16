# Text/I18n Substrate Inventory

Status: phase complete; inventory classifications are terminal and final validation/review evidence is attached.

Platform contract: [platform_contract.md](../platform/platform_contract.md)

CPython checkout: `../cpython` at `14cbd0e6afa98355bdc6749b8230fed4c9b21bd6`.

## Product Boundary

Sifr production text APIs are native substrate APIs, not CPython module clones:

| Surface | Owner milestone | Terminal state | Stability | Notes |
| --- | --- | --- | --- | --- |
| `sifr.encoding.Encoding` | TCP | `production-public` | `stable-public-api` | Static enum/label value for accepted Tier 0/Tier 1 encodings. |
| `sifr.encoding.encode` / `decode` | TCP | `production-public` | `stable-public-api` | Explicit byte/text boundary with typed errors and recovery outcomes. |
| `sifr.encoding.Encoder` / `Decoder` | TCP | `production-public` | `stable-public-api` | Unique mutable incremental state; finalization transitions to exhausted. |
| `str.encode(...)` / `bytes.decode(...)` | TCP | `production-public` | `compiler-known-intrinsic` | Lower through same static registry as `sifr.encoding`. |
| `sifr.io.open_text` / `TextReader` / `TextWriter` | TCP | `production-public` | `stable-public-api` | Explicit encoding required; no locale-derived defaults. |
| `open(..., encoding=..., errors=...)` | TCP | `production-public` | `compiler-known-intrinsic` | Literal/static mode required for binary-vs-text handle type selection. |
| `sifr.unicode.normalize` / `is_normalized` | TLS | `production-public` | `stable-public-api` | NFC/NFD/NFKC/NFKD through selected Unicode data stack. |
| `sifr.unicode` property APIs | TLS | `production-public` | `stable-public-api` | Names, categories, bidi, combining, width, mirrored, decomposition, numeric values. |
| `sifr.unicode.case_fold` | TLS | `production-public` | `stable-public-api` | Locale-sensitive case mapping deferred. |
| `sifr.unicode.graphemes` / `words` and index/boundary variants | M2.5 | `production-public` | `stable-public-api` | Owned iterators; streaming segmentation deferred. |
| `sifr.i18n.LocaleId` / `host_locale` | URL/HTTP primitives | `production-public` / `host-limited` | `stable-public-api` | Host query is read-only and cannot provide default text encoding. |
| `NumberFormatter`, `DateTimeFormatter`, `PluralRules`, `Collator` | URL/HTTP primitives | `production-public` | `stable-public-api` | Object-scoped locale formatting through ICU4X components. |
| `Bundle`, `Message`, `Translator` | HTTP transport | `production-public` | `stable-public-api` | Explicit fallback chains and plural/context lookup. |
| `.mo` catalog loader | HTTP transport | `compat-adapter` | `compatibility-adapter` | Backend/import format behind native bundle API. |
| `sifr.codecs`, `sifr.encodings`, `sifr.unicodedata`, `sifr.locale`, `sifr.gettext` | adapter phase | `deferred-to-phase-adapter` | `compatibility-adapter` | Not production API centers in this phase. |
| bare `codecs`, `encodings`, `unicodedata`, `locale`, `gettext` imports | M0 | `unsupported-with-diagnostic` | `compiler-known-intrinsic` | `SIFR-IMPORT-0008` suggests native `sifr.*` APIs after normal resolution fails. |

## Unsupported And Deferred Python-Shaped Surfaces

| Surface | Owner milestone | Terminal state | Stability | CPython evidence | Regression fixture / revisit |
| --- | --- | --- | --- | --- | --- |
| `codecs.register` | TCP | `unsupported-with-diagnostic` | `compiler-known-intrinsic` | `Lib/codecs.py`, `Lib/test/test_codecs.py::CodecsModuleTest.test_register` | M1 unsupported registry-mutation fixture; bare root covered by `bare_cpython_text_i18n_imports.sifr`. |
| `codecs.unregister` | TCP | `unsupported-with-diagnostic` | `compiler-known-intrinsic` | `Lib/codecs.py`, `Lib/test/test_codecs.py::CodecsModuleTest.test_unregister` | M1 unsupported registry-mutation fixture. |
| `codecs.register_error` | TCP | `unsupported-with-diagnostic` | `compiler-known-intrinsic` | `Lib/codecs.py`, `Lib/test/test_codecs.py` error-handler families | M1 dynamic error-handler fixture. |
| `codecs.open`, `EncodedFile`, `StreamReader`, `StreamWriter`, `StreamReaderWriter`, `StreamRecoder` | TCP | `deferred-to-phase-adapter` | `compatibility-adapter` | `Lib/codecs.py`, `Lib/test/test_codecs.py` stream classes | Revisit only in adapter phase over `sifr.io.open_text`; bare root covered by `bare_cpython_text_i18n_imports.sifr`. |
| public `encodings.*` modules | TCP | `deferred-to-phase-adapter` | `compatibility-adapter` | `Lib/encodings/*.py`, `Lib/encodings/aliases.py` | `bare_cpython_encodings_import.sifr`; dotted root covered by `bare_cpython_dotted_codecs_import.sifr` pattern and future `encodings.*` fixture. |
| CPython codec C APIs | TCP | `rejected` | `test-only-harness` | `Lib/test/test_capi/test_codecs.py`, `Modules/_codecsmodule.c` | No Sifr external CPython C-extension ABI in this phase. |
| `locale.setlocale` | URL/HTTP primitives | `unsupported-with-diagnostic` | `compiler-known-intrinsic` | `Lib/locale.py`, `Lib/test/test_locale.py::TestRealLocales.test_getsetlocale_issue1813` | `bare_cpython_locale_import.sifr`; M3 direct API negative fixture if an adapter root is added. |
| `localeconv` | URL/HTTP primitives | `unsupported-with-diagnostic` | `compiler-known-intrinsic` | `Lib/locale.py`, `Lib/test/test_locale.py` formatting tests | Use object-scoped `NumberFormatter`; M3 negative fixture. |
| `strcoll` / `strxfrm` | URL/HTTP primitives | `deferred-to-phase-adapter` | `compatibility-adapter` | `Lib/locale.py`, `Lib/test/test_locale.py::TestCollation` | Native `Collator` is production API; adapter revisit requires no process-global locale. |
| implicit preferred text encoding | M1/M3 | `rejected` | `compiler-known-intrinsic` | `Lib/locale.py`, `Lib/test/test_locale.py::TestMiscellaneous.test_getpreferredencoding` | M1 missing-encoding `open(...)` diagnostics. |
| `gettext.install` and global `_` | HTTP transport | `unsupported-with-diagnostic` | `compiler-known-intrinsic` | `Lib/gettext.py`, `Lib/test/test_gettext.py::MiscTestCase` | `bare_cpython_gettext_import.sifr`; M4 direct negative fixture if adapter root is added. |
| `gettext.textdomain` / `bindtextdomain` and module-global domain functions | HTTP transport | `deferred-to-phase-adapter` | `compatibility-adapter` | `Lib/gettext.py`, `Lib/test/test_gettext.py::GettextTestCase2` | Native `Bundle`/`Translator` fallback chains are production API. |

## Encoding Tier Decisions

| Tier | State | Encodings |
| --- | --- | --- |
| Tier 0 | required M1 | `utf-8`, `utf-8-sig`, `ascii`, exact `latin-1`, `utf-16-le`, `utf-16-be` |
| Tier 1 | required M1 where covered by `encoding_rs` | `windows-1252` and selected WHATWG `windows-125x` labels |
| Tier 2 | `deferred-to-phase-encoding-expansion` | `utf-32-le`, `utf-32-be`, `Shift_JIS`, `EUC-JP`, `GBK`, `GB18030`, `Big5`, `EUC-KR` |
| Tier 3 | `rejected` / `deferred-to-adapter-phase` | CPython-only aliases, text-to-text codecs, bytes-to-bytes pseudo-codecs, public `encodings.*` parity |

Accepted static aliases are canonical labels plus WHATWG labels covered by `encoding_rs`. CPython-only alias compatibility is rejected unless the alias is also a WHATWG label for an accepted encoding.

### Exact M1 Alias Table

| Encoding | Tier | Accepted labels | Justification |
| --- | --- | --- | --- |
| `Utf8` | Tier 0 | `utf-8`, `utf8`, `utf_8`, `u8` | Canonical plus common CPython/WHATWG-compatible UTF-8 labels. |
| `Utf8Sig` | Tier 0 | `utf-8-sig`, `utf8-sig`, `utf_8_sig`, `utf8_sig` | BOM-aware UTF-8 boundary; CPython-compatible names accepted as static labels. |
| `Ascii` | Tier 0 | `ascii`, `us-ascii`, `us_ascii` | Exact ASCII; web labels that WHATWG maps to Windows-1252 are not reused here unless explicitly listed. |
| `Latin1` | Tier 0 | `latin-1`, `latin1`, `latin_1`, `iso-8859-1`, `iso8859-1` | Exact byte-to-codepoint Latin-1 for file/catalog boundaries. |
| `Utf16Le` | Tier 0 | `utf-16-le`, `utf16le`, `utf_16_le`, `utf-16le` | Explicit little-endian UTF-16 without implicit BOM guessing. |
| `Utf16Be` | Tier 0 | `utf-16-be`, `utf16be`, `utf_16_be`, `utf-16be` | Explicit big-endian UTF-16 without implicit BOM guessing. |
| `Windows1250` | Tier 1 | `windows-1250`, `windows_1250`, `cp1250` | WHATWG/`encoding_rs` label family. |
| `Windows1251` | Tier 1 | `windows-1251`, `windows_1251`, `cp1251` | WHATWG/`encoding_rs` label family. |
| `Windows1252` | Tier 1 | `windows-1252`, `windows_1252`, `cp1252` | Required Tier 1 web/file compatibility label family. |
| `Windows1253` | Tier 1 | `windows-1253`, `windows_1253`, `cp1253` | WHATWG/`encoding_rs` label family. |
| `Windows1254` | Tier 1 | `windows-1254`, `windows_1254`, `cp1254` | WHATWG/`encoding_rs` label family. |
| `Windows1255` | Tier 1 | `windows-1255`, `windows_1255`, `cp1255` | WHATWG/`encoding_rs` label family. |
| `Windows1256` | Tier 1 | `windows-1256`, `windows_1256`, `cp1256` | WHATWG/`encoding_rs` label family. |
| `Windows1257` | Tier 1 | `windows-1257`, `windows_1257`, `cp1257` | WHATWG/`encoding_rs` label family. |
| `Windows1258` | Tier 1 | `windows-1258`, `windows_1258`, `cp1258` | WHATWG/`encoding_rs` label family. |

### Reserved Diagnostics

| Code | Condition | Message | Help |
| --- | --- | --- | --- |
| `SIFR-IO-0801` | Statically visible text-mode `open(...)` without `encoding=` | `text-mode open requires an explicit encoding; Sifr does not use locale-derived default encodings` | `add encoding=Encoding.Utf8 or call sifr.io.open_text(path, encoding=...)` |
| `SIFR-IO-0802` | Nonliteral/dynamic `open(...)` mode string | `open mode must be a string literal so Sifr can choose a binary or text handle type` | `use a literal mode or a future typed helper API` |
| `SIFR-ENCODING-0803` | Dynamic `errors=` handler string for encode/decode/text I/O | `encoding error handlers must be statically known typed values` | `use DecodeErrorHandler.* or EncodeErrorHandler.*` |

## Dependency Decision Records

| Area | Decision | Version | Feature flags | State |
| --- | --- | --- | --- | --- |
| Web encodings | `encoding_rs` | `0.8.35` | default `alloc`; no CJK fast legacy encode features in M1 | accepted for M1 |
| Unicode normalization | `unicode-normalization` | `0.1.25` | default `std` | accepted for M2 |
| Unicode segmentation | `unicode-segmentation` | `1.13.3` | default | accepted for M2.5; MSRV 1.85 is compatible with local Rust 1.94 |
| Locale IDs | `icu_locale` / `icu_locale_core` | `2.2.0` | default off unless required by selected components; compiled data through component crates | accepted for M3 |
| Number formatting | `icu_decimal` + `icu_decimal_data` | `2.2.0` | `compiled_data` | accepted for M3 |
| Date/time formatting | `icu_datetime` + data | `2.2.0` | `compiled_data`, `ixdtf` only if required by formatter input type | accepted for M3 |
| Plural rules | `icu_plurals` + data | `2.2.0` | `compiled_data` | accepted for M3/M4 |
| Collation | `icu_collator` + data | `2.2.0` | `compiled_data` | accepted for M3 |
| Translation catalogs | local audited `.mo` parser | n/a | n/a | accepted for M4 |
| Fluent / ICU message format | none | n/a | n/a | `deferred-to-phase-message-format-backends` |

M1-M4 must hide crate types behind Sifr-owned runtime/compiler API shapes, map all crate errors into typed Sifr errors, and scan emitted/generated runtime code for data-dependent `unwrap`, `expect`, or `panic`.

Detailed decision records live in [text_i18n_dependency_decisions.md](./text_i18n_dependency_decisions.md). Each record includes Sifr abstraction boundaries, Unicode/version alignment, panic/unsafe audit scope, typed error mapping, license/MSRV/binary-size/platform impact, deterministic local test strategy, and supply-chain signal.

## No Global State Policy

Text/i18n follows the shared platform no-global-state policy in [platform_contract.md](../platform/platform_contract.md). The static codec registry is immutable, locale-sensitive APIs are object-scoped, host locale discovery is read-only, `gettext.install`/global `_` mutation is unsupported, and no locale query can legalize text I/O without an explicit `encoding=`.

## Typed Error And Recovery Decisions

Accepted decode handlers: `Strict`, `Replace`, `Ignore`, `BackslashReplace`.

Accepted encode handlers: `Strict`, `Replace`, `Ignore`, `BackslashReplace`, `XmlCharRefReplace`, `NameReplace`.

Dynamic handler names are unsupported. Low-level recoverable operations return `DecodeOutcome { text, recoveries }` or `EncodeOutcome { bytes, recoveries }`. Strict failures return typed errors without partial success. `surrogateescape` and `surrogatepass` are rejected for normal `str` APIs; byte-preserving text boundary types are deferred.

## Binary File I/O Prerequisite

M0 smoke commands passed:

- `cargo run -q -p sifr -- run demos/binary_files/main.sifr`
- `cargo run -q -p sifr -- run demos/bytes_file_io/main.sifr`

Existing coverage also records in-memory `StringIO`/`BytesIO` seek/tell and use-after-close behavior in `crates/sifr/tests/e2e/pass/in_memory_streams.sifr`. File-handle seek/tell remains explicitly unsupported where not implemented; M1 text-mode work must not duplicate or work around file-position behavior.

## Milestone Backlog

| Milestone | Concrete backlog |
| --- | --- |
| TCP | Add `sifr.encoding` static registry, Tier 0/Tier 1 codecs, typed handlers/outcomes, incremental state, `str.encode`, `bytes.decode`, `sifr.io.open_text`, and explicit-encoding `open(...)` lowering. |
| TLS | Add Unicode normalization, property tables, names/lookup, numeric APIs, case folding, and Unicode data version exposure. |
| M2.5 | Add grapheme and word segmentation iterators/boundaries using `unicode-segmentation`. |
| URL/HTTP primitives | Add `LocaleId`, canonicalization, read-only `host_locale`, object-scoped number/date/plural/collation APIs using ICU4X compiled data. |
| HTTP transport | Add native translation bundle/translator API, `.mo` parser, safe plural-expression parser, fallbacks, contexts, plural lookup, missing-key fallback, and missing-path errors. |
| handoff | Add public/internal docs, demos, dependency snapshots, panic scans, final inventory closure, final review, and full validation. |

## M5 Closure Evidence

| Requirement | Evidence |
| --- | --- |
| Public docs | `docs/text_i18n.md` documents `sifr.encoding`, `sifr.unicode`, explicit `sifr.io` text I/O, `sifr.i18n`, and Python-shaped differences. |
| Internal architecture | `internal_docs/architecture.md` records the architecture contract, and `internal_docs/text_i18n_architecture.md` carries the focused substrate closeout note. |
| Demos | `demos/text_i18n/main.sifr` covers non-UTF-8 encode/decode, explicit text open, normalization/properties, segmentation, locale formatting, and translation fallback/plurals. |
| Dependency snapshots | `verification/areas/stdlib_parity/data/text_i18n_dependency_snapshots.json` records generated Cargo dependency snapshots for each text/i18n module and every pairwise/full module combination; `crates/sifr_stdlib/src/features.rs::text_i18n_feature_dependency_snapshots_cover_feature_combinations` locks the same combinations in unit tests. |
| Panic/emitted-code scans | `verification/areas/generated_code_quality/data/corpus_manifest.json` includes `demos/text_i18n/main.sifr` plus representative encoding, Unicode, segmentation, locale, and translation e2e fixtures. |
| E2E fixture manifests | `verification/areas/core_language/data/create_pr_e2e_manifest.json` and `verification/areas/core_language/data/merge_e2e_manifest.json` include all M1-M4 text/i18n pass fixtures as representatives. |
| External review | `reviews/ad-hoc-production-text-i18n-m5-implementation-review-pass-1.md`, `reviews/ad-hoc-production-text-i18n-m5-implementation-review-pass-2.md`, `reviews/ad-hoc-production-text-i18n-m5-implementation-review-pass-3.md`, and `reviews/ad-hoc-production-text-i18n-final-implementation-review-pass-1.md` returned `PASS`; no re-review required. |
| Reference closure | `verification/areas/stdlib_parity/reports/text_i18n_reference_matrix.md` maps CPython codecs, encodings, unicodedata, locale, and gettext families to terminal Sifr dispositions and fixtures. |

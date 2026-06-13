# Ad Hoc Phase: Production Text, Unicode, Encoding, And I18n Runtime

Status: complete
Phase placement: first implementation phase in the split production-stdlib substrate sequence, after the stdlib boundary refactor and before the network/HTTP and concurrency/runtime phases consume text-dependent behavior.
Phase owner: stdlib/runtime implementation with compiler import, file/text I/O, effect, async-workload, and codegen support

## Objective

Build the production-grade text, Unicode, encoding, and i18n substrate required by real Sifr programs and by later web, file I/O, subprocess, and framework phases.

This phase is not a mandate to clone CPython's `codecs`, `encodings`, `unicodedata`, `locale`, or `gettext` modules. CPython sources and tests are used as reference material for behavior, edge cases, fixtures, and waiver evidence. They are not the product API target.

The required output is:

- valid-Unicode Sifr string invariants aligned with Rust `String`/`str`
- explicit byte/text boundary APIs
- static, typed encoding support for production-relevant encodings
- explicit text I/O with required encoding selection
- streaming encode/decode primitives
- Unicode normalization and property APIs
- Unicode segmentation and case folding APIs
- typed locale identifiers
- locale-sensitive formatting through object-based, non-global APIs
- explicit translation bundles and fallback chains
- typed errors
- no user-triggerable emitted Rust panics
- no unsynchronized process-global mutation

CPython-shaped surfaces may be added later only as adapters over this substrate, and only when they are production-useful, maintainable, and compatible with Sifr's static typed model. This phase does not add backward-compatibility support, legacy aliases, deprecated behavior, implicit locale-default text behavior, compatibility shims, hidden bridge names, or bare CPython stdlib import aliases.

## Related Phases

- Network/HTTP platform substrate remains in [ad-hoc-production-network-http-platform-substrate.md](./ad-hoc-production-network-http-platform-substrate.md).
- Concurrency/runtime substrate is tracked in [ad-hoc-production-concurrency-runtime-platform-substrate.md](./ad-hoc-production-concurrency-runtime-platform-substrate.md).
- This phase is the first phase in the split production-stdlib sequence because it provides the shared encoding/text substrate needed by both the network/HTTP and concurrency/runtime phases.
- This phase provides the encoding/text substrate needed by subprocess text mode, HTTP text decoding, file `open(..., encoding=...)`, warning formatting, locale-aware formatting, and translation demos.
- This phase assumes [ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md](./ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md) is complete: Sifr stdlib remains publicly imported through `sifr.*`, and bare CPython stdlib names are not aliases.
- This phase creates and consumes the shared platform contract in [ad-hoc-production-stdlib-platform-contract.md](./ad-hoc-production-stdlib-platform-contract.md). Text M0 must establish that contract's terminal states, stability levels, ownership/lifetime rules, cancellation/backpressure semantics, typed error nesting, observability fields, supported-host matrix, security/resource ownership table, and cross-phase golden fixture manifest before M1 opens.

## Cross-Phase Dependency Contract

The split phase order is explicit:

1. Text/Unicode/encoding/i18n runtime.
2. Concurrency/process/runtime substrate.
3. Network/HTTP platform substrate.

This phase is first because both later phases consume the same text, encoding, Unicode, locale, and explicit text I/O decisions. Concurrency/runtime starts after this phase so process/subprocess and diagnostics work consume the completed text substrate instead of inventing local encoding behavior. Network/HTTP starts last so network/server work consumes both this phase and the production task, cancellation, shutdown, offload, diagnostics, and process model from concurrency/runtime.

- Concurrency/runtime text-dependent surfaces, including subprocess `text=True`, `encoding=...`, `errors=...`, warning output encoding, locale-sensitive warning formatting, and demos that require text-mode `open`, consume this phase's M1/M3 outputs.
- Network/HTTP text-dependent surfaces, including non-UTF-8 HTTP body decoding, URL percent-encoding variants requiring encoding lookup, Unicode/IDNA alignment, file/text handlers, locale-sensitive diagnostics, and text-heavy demos, consume this phase's M1/M2/M2.5/M3 outputs.
- Binary-mode file I/O is prior stdlib/runtime infrastructure owned by the earlier runtime/file-object parity work (`issues/archive/ad-hoc-runtime-and-file-object-parity-expansion.md`) and the current `sifr.io` surface. M0 must verify that binary `open()`/file handles are usable before text-mode integration starts.
- Before M0 starts, run a binary file I/O smoke check covering binary `open`, read, write, seek/tell where supported, close/drop, error-on-use-after-close, and byte-preserving round trips. If it fails, M0 may record inventory only; `milestone_text_i18n_1` is blocked until the stdlib/runtime file-object owner fixes `sifr.io` in a prerequisite PR recorded in this phase's execution ledger. Text-mode code must not work around or duplicate broken binary I/O behavior.
- This phase owns only text-mode integration through explicit text-reader/text-writer APIs and `open(..., encoding=..., errors=...)` lowering.
- Text-mode `open(path)`, `open(path, mode="r")`, and any other text mode without an explicit `encoding=` are unsupported in this phase and remain unsupported after M3. Sifr requires explicit text encodings to preserve static behavior; M3 documents this as an intentional difference from CPython's locale-derived default.
- Python-shaped text stream wrappers such as `io.TextIOWrapper(binary_stream, ...)` are not implemented in this phase. They are recorded as `unsupported-with-diagnostic` with CPython evidence; the production surface is `sifr.io.open_text(...)` and Sifr-native typed text readers/writers.
- In-memory text streams such as `io.StringIO` are encoding-free because they operate on native strings. This phase does not implement or modify `io.StringIO`; M0 only verifies that existing `StringIO` behavior is not incorrectly routed through encoding-backed text wrappers.
- `open(...)` mode strings must be statically known for the compiler to choose binary versus text handle types. Nonliteral/dynamic mode strings are unsupported unless routed through a future typed helper API; this avoids false-positive encoding diagnostics for dynamic binary modes and avoids a handle type that depends on runtime string contents.
- Statically visible text-mode opens without `encoding=` produce a compile-time diagnostic requiring `encoding=...`. Dynamic/nonliteral mode opens produce a compile-time diagnostic requiring a literal mode or typed helper. Sifr must not silently substitute UTF-8 for CPython's locale-derived default.
- Consumers must call the encoding/text substrate owned by this phase; they must not add local encoding fallbacks.

## Source Of Truth

The authoritative CPython source tree for reference scanning is configured through `SIFR_CPYTHON_CHECKOUT`. For this planning run, that checkout is:

- `/Users/yaseralnajjar/work/sifr/cpython`

The implementation must scan and classify these CPython files before each milestone implementation PR, but the scan classifies surfaces as production substrate, future adapter, host-limited, rejected, or deferred. It does not automatically promote Python-shaped APIs into Sifr production APIs.

| Domain | CPython library sources | CPython test sources | Native backing sources |
| --- | --- | --- | --- |
| codecs/encodings | `Lib/codecs.py`, `Lib/encodings/*.py`, `Doc/library/codecs.rst` | `Lib/test/test_codecs.py`, `Lib/test/test_capi/test_codecs.py` | `Modules/_codecsmodule.c`, `Modules/cjkcodecs/*` |
| Unicode data | `Doc/library/unicodedata.rst` | `Lib/test/test_unicodedata.py` | `Modules/unicodedata.c` |
| locale/gettext | `Lib/locale.py`, `Lib/gettext.py`, `Doc/library/locale.rst`, `Doc/library/gettext.rst` | `Lib/test/test_locale.py`, `Lib/test/test_gettext.py` | `Modules/_localemodule.c` |

Path note: CPython paths above are relative to `SIFR_CPYTHON_CHECKOUT`; the local planning checkout is `/Users/yaseralnajjar/work/sifr/cpython`.

Standards and Rust ecosystem candidates must also be reviewed before the affected milestone starts:

- Rust `String`/`str` UTF-8 and character-boundary invariants
- WHATWG Encoding Standard and `encoding_rs`
- Unicode data, normalization, case mapping, and segmentation standards
- `unicode-normalization`, `unicode-segmentation`, and generated table options
- ICU4X/ICU-style APIs for locale IDs, formatting, plural rules, collation, and segmentation
- Fluent and gettext catalog formats as translation backends

## Current Sifr Baseline

- `sifr.io` has file handles and in-memory stream wrappers, but no codec-driven text stream layer beyond current UTF-8-oriented boundaries.
- Binary-mode file I/O is existing `sifr.io` infrastructure and remains outside this phase except where text-mode wrappers compose over it.
- `str.encode(...)`, `bytes.decode(...)`, and `open(..., encoding=...)` do not have production encoding substrate support.
- `sifr.encoding`, `sifr.unicode`, and `sifr.i18n` are not present as production stdlib surfaces.
- Python-shaped `sifr.codecs`, `sifr.encodings`, `sifr.unicodedata`, `sifr.locale`, and `sifr.gettext` are not production surfaces for this phase.

The Phase 32 async/workload model remains binding:

- CPU-heavy table generation, normalization, segmentation, collation, formatting, or large codec work must be classified as `@cpu_heavy` where appropriate.
- Blocking file reads for `.mo` catalogs, Fluent resources, or locale data must be classified as `@blocking_io`.
- Direct calls to blocking or CPU-heavy sync APIs from `async def` remain compiler errors unless routed through native async APIs or explicit offload.

## Support Tiers

| Tier | Meaning | Examples |
| --- | --- | --- |
| Production substrate | Required for real programs and later phases | valid text invariants, byte/text boundaries, encoding, text I/O, Unicode normalization, Unicode properties, segmentation |
| Production API | Recommended user-facing Sifr API | `sifr.encoding`, `sifr.unicode`, `sifr.io`, `sifr.i18n` |
| Import/compatibility backend | Useful for migration/import but not the main API | `.mo` gettext loader, selected CPython-shaped wrappers in a later adapter phase |
| Host-limited | Depends on platform data or OS behavior | host preferred encoding queries, OS locale-name discovery |
| Rejected/deferred | Too dynamic, global, legacy, or costly for this phase | `setlocale`, `codecs.register`, `gettext.install`, full `encodings.*` parity |

Authoritative terminal states, evidence states, and stability levels come from [ad-hoc-production-stdlib-platform-contract.md](./ad-hoc-production-stdlib-platform-contract.md). The inventory state `open` is forbidden at phase exit.

## No-Toy-Module Gate

A public text/i18n module may be added only if it satisfies at least one of:

1. It is necessary production substrate.
2. It is the recommended production developer API.
3. It is a stable, broadly useful utility with low long-term maintenance cost.
4. It is required by file I/O, subprocess text mode, HTTP text decoding, diagnostics, Phase 41, packaging, or localization work.

The following are not sufficient reasons:

- CPython has the module.
- It helps a compatibility demo.
- It is easy to partially implement.
- It can be marked as basic and fixed later.
- It mirrors an old dynamic/global API shape.

Partial public modules are rejected unless they are explicitly unstable/internal and inaccessible as stable user APIs.

## Public API Policy

The production public API center is:

- `sifr.encoding`
  - `Encoding`
  - `decode(bytes, encoding, errors=...)`
  - `encode(text, encoding, errors=...)`
  - `Decoder`
  - `Encoder`
  - `DecodeError`
  - `EncodeError`
  - typed decode and encode error handlers
- `sifr.unicode`
  - `normalize(text, Normalization.NFC | NFD | NFKC | NFKD)`
  - `is_normalized(text, normalization)`
  - `category(char)`
  - `bidirectional(char)`
  - `combining(char)`
  - `east_asian_width(char)`
  - `mirrored(char)`
  - `decomposition(char)`
  - `name(char)`
  - `lookup(name)`
  - `decimal(char)`
  - `digit(char)`
  - `numeric_value(char)`
  - `case_fold(text)`
  - `graphemes(text)`
  - `grapheme_indices(text)`
  - `words(text)`
  - `word_boundaries(text)`
  - sentence boundaries are deferred out of this phase
- `sifr.io`
  - `open_text(path, encoding=Encoding.Utf8, errors=DecodeErrorHandler.Strict)`
  - `TextReader`
  - `TextWriter`
  - `TextDecodeError`
  - `TextEncodeError`
  - `open(..., encoding=...)` lowering to the same typed text I/O substrate
- `sifr.i18n`
  - `LocaleId`
  - locale parsing and canonicalization
  - `NumberFormatter`
  - `DateTimeFormatter`
  - `Collator`
  - `PluralRules`
  - `Bundle`
  - `Message`
  - `Translator`
  - explicit fallback chains

The following are not production API centers in this phase:

- `sifr.codecs`
- `sifr.encodings`
- public `sifr.encodings.*` modules
- `sifr.unicodedata`
- `sifr.locale`
- `sifr.gettext`

If any Python-shaped module is implemented later, it must be a thin adapter over the Sifr-native substrate, recorded as `compat-adapter` after acceptance or `deferred-to-adapter-phase` before acceptance, and reviewed for static typing, ownership, no-global-state, and panic-free behavior. Bare CPython module imports such as `import codecs`, `import locale`, or `import gettext` are not aliases for `sifr.*`; they receive the namespace-contract diagnostic after normal user/package resolution fails.

## Milestone Dependency Graph

1. `milestone_text_i18n_0` first. No implementation milestone starts until the product boundary, Rust lowering contract, API names, support tiers, CPython classification matrix, Unicode version decision, encoding tiers, and no-global-state policy are checked in.
2. `milestone_text_i18n_1` before all consumers. Encoding and explicit text I/O are the substrate for file I/O, subprocess text mode, HTTP text decoding, and translation catalog parsing.
3. `milestone_text_i18n_2` can run after M0 but must lock the Unicode data version before normalization/property APIs ship.
4. `milestone_text_i18n_2_5` can run after M0 and should share the Unicode version/data strategy with M2.
5. `milestone_text_i18n_3` waits for M0 and may use M1 only for host encoding queries or catalog loading; it must not introduce locale-derived default text I/O.
6. `milestone_text_i18n_4` waits for M1 for `.mo` file decoding and declared catalog encodings.
7. `milestone_text_i18n_5` closes docs, demos, validation, review, and waivers last.

## Architecture Principles

### Valid Text Invariants

Sifr text APIs lower to Rust UTF-8 `String`/`str`-like invariants:

- Normal Sifr strings are always valid Unicode scalar text.
- Arbitrary bytes are bytes, not text.
- Decoding is an explicit boundary operation.
- Encoding is an explicit boundary operation.
- APIs that decode arbitrary bytes must either return valid Sifr text, return typed decode errors, or return an explicit recovery-carrying value.
- Invalid byte sequences, lone surrogates, partial decodes, and recovery behavior must not be hidden inside ordinary strings.
- Surrogate-preserving behavior must be deferred or isolated behind a special boundary type such as `BytePreservingText`, `EscapedText`, or an equivalent explicit value. It must not smuggle invalid Unicode into normal `str`.

### Native Substrate First, Python Adapters Later

Implement the canonical Sifr text primitive first. CPython-shaped modules are not baseline product scope.

- A private runtime registry owns encoding lookup, encoder/decoder construction, static aliases, and typed handler mapping.
- `sifr.encoding`, `str.encode(...)`, `bytes.decode(...)`, `sifr.io.open_text(...)`, and `open(..., encoding=...)` must use the same registry to avoid dual semantics.
- Registry mutation is not accepted in this phase. Dynamic codec lookup by name is supported only against the static registry.
- `codecs.register`, `codecs.unregister`, `codecs.register_error`, dynamic error-handler lookup, `codecs.open`, `EncodedFile`, `StreamReader`, `StreamWriter`, `StreamReaderWriter`, and full `CodecInfo` compatibility are not required phase outputs.

### Rust Ecosystem First

This phase builds a Sifr text/i18n platform, not a new Unicode or locale stack. The implementation should wrap, constrain, and test mature Rust ecosystem crates wherever they satisfy Sifr's static typing, ownership, binary-size, license, Unicode-version, panic-free, and maintenance requirements.

M0 must produce a dependency decision record for every crate family below. Each decision must include accepted crate and feature flags, Sifr abstraction that hides crate types from public APIs, Unicode/version alignment, panic/unsafe audit for user-controlled text and bytes, typed error mapping into Sifr variants, license/MSRV/binary-size/platform impact, deterministic local test strategy, and supply-chain/maintenance signal.

| Area | Preferred crate families | Role |
| --- | --- | --- |
| Web-compatible encoding labels and legacy web encodings | `encoding_rs` | WHATWG-compatible decoding/encoding for HTTP/HTML/file boundaries, including Windows-125x labels covered by Tier 1. Tier 2 CJK encodings are excluded from this phase. |
| Unicode normalization | `unicode-normalization` | NFC/NFD/NFKC/NFKD and normalization checks with the selected Unicode data version. |
| Unicode segmentation | `unicode-segmentation` | grapheme and word boundaries only. Sentence boundaries are deferred. |
| Unicode properties and generated data | generated tables first, with focused crates or ICU4X components only when they reduce table ownership without version skew | names, categories, numeric values, widths, bidi data, case mapping, and versioned property lookup. |
| Locale identifiers, formatting, plural rules, collation | ICU4X `icu` components | typed locale IDs, number/date formatting, plural rules, collation, and locale data without process-global mutation. |
| Translation catalog parsing | local audited `.mo` parser over the M1 encoding substrate | deterministic gettext catalog import as a backend format, not the primary i18n API. |
| Modern message formatting | deferred | Fluent and ICU message-format backends are not implemented in this phase. The `sifr.i18n` bundle API leaves room for a later backend. |

From-scratch encoding algorithms, Unicode normalization, segmentation, locale formatting, plural rules, collation, or message-format engines are rejected in this phase. If the selected Rust ecosystem stack cannot satisfy a required surface, that surface is deferred with evidence instead of receiving a bespoke implementation. Generated tables are allowed where the data is the product surface, but generation must be reproducible, reviewed, marked as generated, and paired with a checked-in regeneration command.

### Encoding Support Tiers

Encoding support is demand-tiered:

| Tier | Required status | Encodings |
| --- | --- | --- |
| Tier 0 | Required for phase exit | `utf-8`, `utf-8-sig`, `ascii`, exact `latin-1`, `utf-16-le`, `utf-16-be` |
| Tier 1 | Required for phase exit through `encoding_rs` where covered | `windows-1252`, WHATWG label decoding for HTTP/HTML boundaries, and selected `windows-125x` labels covered by `encoding_rs` |
| Tier 2 | Deferred to a future feature phase | `utf-32-le`, `utf-32-be`, `Shift_JIS`, `EUC-JP`, `GBK`, `GB18030`, `Big5`, `EUC-KR` |
| Tier 3 | Deferred/rejected for this phase | obscure CPython-only aliases, text-to-text codecs, bytes-to-bytes pseudo-codecs, full public `encodings.*` module parity |

M0 must record the exact Tier 0/Tier 1 alias table and any binary-size, license, or generated-table constraints before M1 starts. Tier 2 encodings are not accepted for this phase.

### Typed Error Handlers

Encoding APIs use typed handler values, not dynamic Python-style handler names:

- `DecodeErrorHandler.Strict`
- `DecodeErrorHandler.Replace`
- `DecodeErrorHandler.Ignore`
- `DecodeErrorHandler.BackslashReplace`
- `EncodeErrorHandler.Strict`
- `EncodeErrorHandler.Replace`
- `EncodeErrorHandler.Ignore`
- `EncodeErrorHandler.BackslashReplace`
- `EncodeErrorHandler.XmlCharRefReplace`
- `EncodeErrorHandler.NameReplace`

String-literal compatibility at call sites may be lowered to typed handler values only when statically known and valid for that context. Dynamic `errors=` strings and dynamic handler registration are unsupported in this phase. `surrogateescape` and `surrogatepass` are deferred or isolated behind an explicit byte-preserving boundary type; they are not normal string handlers.

Strict incremental encode/decode failures return typed errors with no successful partial-output value. Recoverable non-strict handlers return typed success outcomes that preserve both produced output and recovery evidence, such as `DecodeOutcome { text, recoveries }` or `EncodeOutcome { bytes, recoveries }`. Convenience layers may expose only produced text/bytes only after the lower runtime contract retains recovery diagnostics for validation and tracing.

### Incremental Codec Ownership

`Encoder` and `Decoder` are stateful linear values:

- encode/decode calls require a unique mutable handle (`&mut`-equivalent in the lowered Rust model)
- the compiler rejects concurrent aliasing of the same incremental codec object
- incremental codec objects are not `Send`/`Sync` and are not shareable across tasks/threads unless a future explicit locked wrapper API is added
- `final=True` transitions the object to an exhausted state; later calls through the same handle return typed exhausted errors
- adapters lower to this same unique-mutable state model rather than `RefCell` or hidden shared mutation

### Unicode Data Versioning

- M0 must record the Unicode data version and the `SIFR_CPYTHON_CHECKOUT` commit used for reference fixtures.
- Generated Unicode tables must be reproducible, reviewed, excluded from the hand-maintained 900-line guardrail where appropriate, marked with a generated-file header, and paired with a checked-in regeneration command.
- Runtime APIs must expose the Unicode data version consistently with the table set used by the build.
- Normalization, properties, case folding, and segmentation must either share the same Unicode version or explicitly record any version skew as a release blocker.

### Locale And I18n Without Process Globals

- Sifr does not clone Python's process-global `locale` model as a production API.
- `setlocale`-style mutation, `localeconv` as a primary API, `strcoll`, `strxfrm`, `format_string`, and `currency` are unsupported in this phase. Any future adapter issue must prove it can expose a Sifr-native, object-scoped model without global mutation hazards.
- Production locale-sensitive behavior uses typed locale identifiers and object-based formatters in `sifr.i18n`.
- Host locale discovery is host-limited and read-only. It must not make implicit text encodings legal.
- Collation, number formatting, date/time formatting, and plural rules use explicit locale values and explicit formatter objects.
- Translation uses explicit bundles, translators, and fallback chains. `gettext.install`-style global mutation and global `_` injection are unsupported.

### Typed Errors Instead Of Exceptions

All fallible APIs must expose typed error results:

- `EncodeError`
- `DecodeError`
- `UnicodeDataError`
- `LocaleIdError`
- `FormatError`
- `PluralRulesError`
- `TranslationError`
- `CatalogError`

Names may align with known terminology where useful, but the operational contract is Sifr `Result`/`Option`, not exception-driven control flow.

### Panic-Free Runtime Contract

Generated Rust for these APIs must not contain data-dependent `.unwrap()`, `.expect()`, or `panic!` on user-controlled text, byte sequences, locale names, format strings, Unicode data names, translation keys, or catalog data.

## Non-Goals And Permanent Boundaries

The following are not accepted as silent omissions. They must be explicitly recorded with diagnostics, tests, and waiver evidence:

- CPython codec registry mutation through `codecs.register`/`codecs.unregister`
- CPython error-handler registry mutation through `codecs.register_error`
- public `encodings.*` module parity
- text-to-text and bytes-to-bytes pseudo-codecs
- deprecated `codecs.open`-style wrappers
- `EncodedFile`, `StreamReader`, `StreamWriter`, and `StreamReaderWriter` as required APIs
- dynamic `errors=` handler names
- `surrogateescape`/`surrogatepass` in normal `str` APIs
- process-global `locale.setlocale` as a production API
- `localeconv`, `strcoll`, and `strxfrm` as primary APIs
- implicit locale-derived text encodings
- `gettext.install` and global `_` mutation
- C API compatibility for external CPython extensions
- CJK and all-codepage parity in this phase; Tier 2 encodings require a separate follow-up issue with dependency review, data-size review, and workload justification
- host-specific locale names that cannot be made deterministic on the supported host matrix

## Milestones

### milestone_text_i18n_0: Product Boundary And Rust Lowering Contract

Scope:

- Add a machine-readable substrate inventory under `verification/stdlib/text_i18n_substrate_inventory.*`.
- Create or update the shared platform contract artifacts: `verification/platform/platform_contract.md`, `verification/platform/platform_contract.json`, `verification/platform/supported_host_matrix.md`, `verification/platform/golden/manifest.json`, and `scripts/run_platform_golden.sh`.
- Obtain an external review `PASS` on the shared platform contract before M1 opens.
- Scan every source/test/doc file listed in `Source Of Truth`.
- Extract public functions, classes, constants, methods, common keyword forms, current codec aliases, encoding module names, deprecation/legacy markers, and test-class/test-method names.
- Classify each extracted CPython surface with the shared platform terminal states and stability levels.
- Add Sifr-native fixture plans for:
  - `sifr_encoding_subset.sifr`
  - `sifr_text_io_subset.sifr`
  - `sifr_unicode_subset.sifr`
  - `sifr_unicode_segmentation_subset.sifr`
  - `sifr_i18n_locale_formatting_subset.sifr`
  - `sifr_i18n_translation_subset.sifr`
- Add negative import-resolution tests for bare CPython stdlib import attempts and avoid positive tests for Python-shaped modules unless a later adapter phase accepts them.
- Define Text versus Bytes invariants and the Rust lowering contract.
- Map text/i18n security/resource concerns, including codec amplification, malformed byte sequences, malicious catalogs, locale discovery, and host-limited formatting, to the shared platform contract.
- Add or update text-owned cross-phase golden fixtures in `verification/platform/golden/manifest.json`.
- Record resolved implementation decisions:
  - public API names under `sifr.encoding`, `sifr.unicode`, `sifr.io`, and `sifr.i18n`
  - Unicode data version
  - generated table strategy
  - supported encoding tiers
  - selected Rust crates and feature flags
  - checked-in dependency decision records for every Rust Ecosystem First crate family
  - typed encode/decode error handlers
  - recovery-carrying value shape
  - surrogate behavior policy
  - host-limited locale discovery matrix
  - diagnostic wording for explicit-encoding-required text `open(...)` and literal-mode-required `open(...)`
  - translation backend scope and `.mo` support decision
- Assign each inventory entry one owner milestone, one shared terminal state, and one stability level.

Definition of done:

- The backlog is derived from CPython sources/tests, Unicode/encoding/i18n standards, and selected Rust runtime crates rather than hand-written memory.
- Every target capability has a first-pass surface matrix and fixture matrix.
- Every CPython-shaped surface has a disposition and is not accidentally treated as required production API.
- Shared platform contract artifacts exist and have an external review `PASS`.
- Every accepted or rejected Rust ecosystem crate family has a checked-in dependency decision record before M1 starts.
- M1-M5 implementation PRs, including M2.5, have concrete backlog entries rather than prose-only scope.

### milestone_text_i18n_1: Encoding And Explicit Text I/O

Scope:

- Provide the exact cross-phase unblock point for:
  - network/web `blocked-on-text-i18n-m1` surfaces: non-UTF-8 URL quoting/parsing forms, HTTP body text decoding, and network demos that require `open(..., encoding=...)`
  - concurrency/runtime `blocked-on-text-i18n-m1` surfaces: subprocess `text=True`, `encoding=...`, `errors=...`, warning output encoding, and demos that require text-mode `open`
  - locale-sensitive warning formatting remains additionally blocked on `milestone_text_i18n_3`
- Add `sifr.encoding`:
  - `Encoding`
  - static encoding labels and aliases from `Resolved M0 Decisions`
  - `encode`
  - `decode`
  - `Encoder`
  - `Decoder`
  - `EncodeError`
  - `DecodeError`
  - `EncodeOutcome`
  - `DecodeOutcome`
  - typed encode/decode error handlers
  - strict and recoverable error behavior
  - incremental finalization and exhausted-state errors
- Add Tier 0 encodings and accepted Tier 1 encodings.
- Record Tier 2 encodings as deferred; do not feature-gate them in this phase.
- Add explicit text I/O integration:
  - `sifr.io.open_text(path, encoding=..., errors=...)`
  - `TextReader`
  - `TextWriter`
  - `open(..., encoding=..., errors=...)` lowering
  - record Python-shaped `io.TextIOWrapper(..., encoding=..., errors=...)` as unsupported in this phase; `sifr.io.open_text` is the production API
  - no-encoding text `open(...)` diagnostics defined by M0
  - no locale-derived default encoding
- Keep `str.encode(...)` and `bytes.decode(...)` routed through the same substrate.
- Keep CPython-shaped codec APIs out of required scope. If a minimal adapter is proposed, it must be separately reviewed and recorded as `future-adapter`, not as the production path.

CPython tests to mine for fixture ideas and waiver evidence:

- `Lib/test/test_codecs.py`
- `Lib/test/test_capi/test_codecs.py`

Rust/runtime candidates:

- `encoding_rs`
- generated tables for encodings not covered by selected crates

Definition of done:

- Static lookup, aliases, BOM handling, incremental encoding/decoding, and typed error handlers pass Sifr-native fixtures.
- Incremental encoder/decoder finalization and post-finalization exhaustion have fixtures for both statically known and dynamic `final` values.
- Strict errors return typed failure with no partial success.
- Recoverable non-strict errors return partial output plus recovery diagnostics.
- `str.encode(encoding, errors)` and `bytes.decode(encoding, errors)` have fixtures for supported typed handlers, unsupported dynamic handler names, and invalid context combinations.
- Text I/O uses the same encoding substrate as explicit encode/decode APIs.
- Static-registry behavior has fixtures for alias resolution, unsupported mutation APIs, and no silent fallback on missing encodings.
- Tier 0/Tier 1/Tier 2 feature-size and dependency decisions are recorded.

### milestone_text_i18n_2: Unicode Core

Scope:

- Add `sifr.unicode` normalization:
  - `normalize`
  - `is_normalized`
  - NFC, NFD, NFKC, NFKD
- Add Unicode properties:
  - `name`
  - `lookup`
  - `category`
  - `bidirectional`
  - `combining`
  - `east_asian_width`
  - `mirrored`
  - `decomposition`
  - `decimal`
  - `digit`
  - `numeric_value`
  - data version
- Add case mapping and matching:
  - `case_fold`
  - simple case mapping where required by `case_fold`; full locale-sensitive case mapping is deferred
- Generate or vendor Unicode data tables according to the M0 version decision.
- Ensure normalization, property queries, and case mapping share the same table version.

CPython tests to mine for fixture ideas and waiver evidence:

- `Lib/test/test_unicodedata.py`

Rust/runtime candidates:

- `unicode-normalization`
- generated Unicode tables for names, properties, widths, bidi data, numeric values, and case mapping
- no ICU4X Unicode-property dependency in M2; ICU4X use starts in M3 for locale/i18n

Definition of done:

- Unicode normalization and property queries pass Sifr-native fixtures adapted from CPython and Unicode data.
- The shipped Unicode data version is exposed consistently.
- Missing-name and missing-property paths return typed errors/options, never panics.
- Python-shaped `sifr.unicodedata` remains deferred unless a separate adapter decision is recorded.

### milestone_text_i18n_2_5: Unicode Segmentation

Scope:

- Add `sifr.unicode` segmentation:
  - `graphemes(text)`
  - `grapheme_indices(text)`
  - `words(text)`
  - `word_boundaries(text)`
  - sentence boundaries are deferred
- Define iterator ownership and allocation behavior.
- Streaming segmentation is deferred; this phase provides owned-string iterators only.
- Ensure segmentation data version aligns with M2 or record version skew as a release blocker.

Rust/runtime candidates:

- `unicode-segmentation`
- no ICU4X segmentation dependency in M2.5

Definition of done:

- Grapheme and word-boundary fixtures cover combining marks, emoji sequences, regional indicators, whitespace, punctuation, and mixed-script text.
- APIs expose user-perceived text boundaries rather than byte offsets alone.
- No API permits slicing Rust strings on invalid UTF-8 or non-boundary byte positions.

### milestone_text_i18n_3: Locale Identifiers And Locale-Sensitive Formatting

Scope:

- Add `sifr.i18n` locale support:
  - `LocaleId`
  - parsing
  - canonicalization
  - likely-subtag behavior through ICU4X
- Add object-based formatting:
  - `NumberFormatter`
  - `DateTimeFormatter`
  - `PluralRules`
  - `Collator`
  - display names are deferred
- Add read-only host locale discovery as `sifr.i18n.host_locale() -> Option[LocaleId]`, mark it `host-limited`, and forbid it from providing text I/O defaults.
- Do not add process-global `setlocale` behavior.
- Do not make locale preferred encoding APIs legalize text `open(...)` without `encoding=`.

CPython tests to mine for anti-requirements, host-limited evidence, and fixture ideas:

- `Lib/test/test_locale.py`

Rust/runtime candidates:

- ICU4X/ICU-style components
- ICU4X locale identifier components

Definition of done:

- Locale identifiers are parsed and canonicalized deterministically.
- Number/date formatting, plural rules, and accepted collation behavior use explicit locale values and formatter objects.
- Supported locale data and host assumptions are documented and tested.
- Python `locale` process-global APIs are recorded as `unsupported-with-diagnostic` or `deferred-to-adapter-phase`, not production surfaces.
- Locale errors are typed and never panic.

### milestone_text_i18n_4: Translation Bundles

Scope:

- Add `sifr.i18n` translation support:
  - `Bundle`
  - `Message`
  - `Translator`
  - explicit fallback chains
  - plural-aware message lookup
  - context-aware message lookup
  - typed arguments where supported by the chosen backend
- Add `.mo` loader/importer as a compatibility backend.
- Integrate `.mo` decoding with the M1 encoding substrate.
- Parse `.mo` plural expressions with a constrained safe plural-expression parser; do not evaluate catalog plural metadata through a general Sifr, Python, shell, or host expression engine.
- Reserve backend-neutral `Bundle`/`Translator` interfaces for future Fluent/ICU message-format support, but do not implement those backends in this phase.
- Do not add `gettext.install` or global `_` mutation.

CPython tests to mine for fixture ideas and waiver evidence:

- `Lib/test/test_gettext.py`

Rust/runtime candidates:

- local audited `.mo` parser and constrained safe plural-expression parser implemented in Sifr runtime/Rust
- no Fluent dependency in this phase

Definition of done:

- Translation parsing is deterministic and panic-free.
- `.mo` plural expressions are parsed by the constrained safe plural-expression parser, reject unsupported constructs with typed catalog errors, and never invoke a general expression engine.
- Plural forms, contexts, fallback chains, missing keys, and missing catalog paths have Sifr-native fixtures.
- `.mo` support is documented as an import/backend format rather than the strategic i18n API.
- Python-shaped `sifr.gettext` remains deferred unless a separate adapter decision is recorded.

### milestone_text_i18n_5: Integration, Documentation, And Production Gate

Scope:

- Update public docs for:
  - `sifr.encoding`
  - `sifr.unicode`
  - `sifr.io` explicit text I/O
  - `sifr.i18n`
  - intentional differences from Python-shaped text APIs
- Update internal architecture docs for:
  - valid text invariants
  - encoding registry and text I/O boundaries
  - generated Unicode table strategy
  - segmentation strategy
  - locale/i18n data strategy
  - translation catalog parsing
  - host-limited behavior
- Add demos:
  - non-UTF-8 encode/decode
  - `open(..., encoding=...)`
  - Unicode normalization/property lookup
  - grapheme/word segmentation
  - locale formatting with explicit `LocaleId`
  - translation bundle with fallback and plural behavior
- Add generated Cargo dependency snapshots for all new feature combinations.
- Add panic-scan and emitted-code quality checks for encoding, Unicode, locale, formatting, and translation paths.
- Update validation lane manifests with representative fixtures.
- Close the inventory:
  - every production API has a terminal state
  - every CPython reference test family has classification evidence
  - every rejected/deferred Python-shaped surface has a revisit rule and regression fixture
  - every host-limited surface records the supported host matrix
- Run an external review loop on the final inventory and close any blocking finding before phase completion.
- External review owner is the stdlib phase owner plus the designated compiler/runtime reviewer recorded in the execution ledger. If review output is unavailable for five working days after the review artifact is posted, the phase owner may proceed only by recording the attempted review, open questions, and a conservative self-review in the ledger.

Validation:

- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- file-size guardrail
- `cargo test -p sifr_stdlib`
- `cargo test -p sifr -- stdlib`
- `scripts/run_e2e_pass.sh`
- `scripts/run_all_tests.sh --profile create-pr`
- `scripts/run_all_tests.sh`

Definition of done:

- Every production API and reference test family in the phase inventory is closed with a shared platform terminal state and stability level.
- No implementation-owned source file exceeds the 900-line guardrail.
- No user-triggerable runtime panic path exists in the added stdlib/runtime surfaces.
- No unsynchronized process-global mutation exists.
- CPython-shaped adapter work is either explicitly deferred or separately accepted with Sifr-native substrate backing.

## Required Tracking Artifacts

Create and keep current during implementation:

- `issues/ad-hoc-production-text-i18n-platform-substrate-execution.md`
- `verification/stdlib/text_i18n_substrate_inventory.md`
- `verification/stdlib/text_i18n_substrate_inventory.json`
- `verification/stdlib/text_i18n_reference_matrix.md`
- one traceability document per milestone domain under `verification/stdlib/`
- `verification/platform/platform_contract.md`
- `verification/platform/platform_contract.json`
- `verification/platform/supported_host_matrix.md`
- `verification/platform/golden/manifest.json`
- `scripts/run_platform_golden.sh`

The execution ledger must record:

- planning/review artifacts
- per-milestone PR links
- local validation commands and results
- CPython source/test files scanned
- standards and Rust crates reviewed
- shared platform terminal states and stability levels
- shared platform golden fixture entries and skip/pass status for text-owned contracts
- shared platform security/resource ownership rows for codec amplification, malformed byte sequences, catalogs, locale discovery, and formatting
- mined-as-substrate-fixture/adapted-for-sifr-api/compat-adapter-deferred/blocked-on-phase-X/external-signal/waived-with-rationale/rejected reference test families
- final unsupported-with-diagnostic/deferred-to-phase-X/host-limited/rejected index

## Quality Contract

- Solve root causes rather than adding workaround wrappers.
- No backward-compatibility shims, legacy aliases, deprecated behavior, implicit locale-default behavior, fallback paths, or CPython-shaped public API commitments may survive phase exit unless separately accepted as adapters over the Sifr-native substrate.
- No unsynchronized process-global state may be introduced.
- No data-dependent emitted `.unwrap()`, `.expect()`, or `panic!` is allowed in user runtime paths.
- Every added CPU-heavy or blocking sync function must be classified in the stdlib workload database.
- Every added external crate dependency must be represented by a stable `StdlibFeature` in `sifr_stdlib`.
- Every production module added to embedded stdlib sources must have canonical `sifr.*` import-resolution tests, type-check tests, e2e pass tests, and negative diagnostics for unsupported bare CPython import forms.

## Resolved M0 Decisions

M0 records evidence for these decisions; it does not reopen them without a new issue.

| Decision area | Decision |
| --- | --- |
| Public API names | Production APIs are `sifr.encoding`, `sifr.unicode`, `sifr.io.open_text`, and `sifr.i18n`. Python-shaped modules remain `deferred-to-adapter-phase`. |
| Encoding tiers | Tier 0 and Tier 1 are required exactly as defined in `Encoding Support Tiers`. Tier 2 encodings are deferred and may not be silently added during M1. |
| Unicode data version | Use the Unicode data version shipped by the selected `unicode-normalization`, `unicode-segmentation`, generated tables, and ICU4X crates. M0 must record the exact version from the locked crate/data set; any version skew across normalization, properties, case mapping, segmentation, and i18n is a release blocker. |
| Static aliases | Ship canonical labels plus WHATWG labels covered by `encoding_rs` for accepted encodings. Obscure CPython-only aliases are rejected unless they are also WHATWG labels. Unsupported registry mutation diagnostics name the static-registry policy and suggest canonical `sifr.encoding` APIs. |
| Error handlers | Accepted decode handlers: `Strict`, `Replace`, `Ignore`, `BackslashReplace`. Accepted encode handlers: `Strict`, `Replace`, `Ignore`, `BackslashReplace`, `XmlCharRefReplace`, `NameReplace`. Dynamic handler names are unsupported. |
| Recovery diagnostics | Recoverable operations return `DecodeOutcome { text, recoveries }` or `EncodeOutcome { bytes, recoveries }` from low-level APIs. Convenience APIs may expose produced text/bytes only after the runtime preserves recovery evidence for tracing and tests. |
| Surrogate behavior | `surrogateescape` and `surrogatepass` are rejected for normal `str` APIs in this phase. Byte-preserving text boundary types are deferred to a separate OS/path interop issue. |
| Crate/data strategy | `encoding_rs`, `unicode-normalization`, `unicode-segmentation`, generated Unicode tables, and ICU4X components are the default stack. Fluent and ICU message-format engines are deferred. |
| Host locale queries | The only host locale query in this phase is read-only `sifr.i18n.host_locale() -> Option[LocaleId]`, marked `host-limited`. It never supplies text I/O defaults. |
| Translation backend | `.mo` loading is accepted as a compatibility backend behind `sifr.i18n.Bundle`/`Translator`. `.mo` plural expressions use a constrained safe plural-expression parser, not a general expression engine. Fluent and ICU message-format backends are deferred; the API must not commit to either backend. |
| Unsupported Python-shaped APIs | `setlocale`, `localeconv`, `strcoll`, `strxfrm`, `codecs.register`, `codecs.unregister`, `codecs.register_error`, `gettext.install`, global `_`, public `encodings.*`, and CPython C APIs receive unsupported/deferred diagnostics with CPython evidence and regression fixtures. |

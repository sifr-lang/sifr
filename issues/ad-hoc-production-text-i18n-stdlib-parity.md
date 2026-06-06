# Ad Hoc Phase: Production Text, Unicode, Encoding, And I18n Runtime

Status: draft
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
- Concurrency/runtime stdlib parity is tracked in [ad-hoc-production-concurrency-runtime-stdlib-parity.md](./ad-hoc-production-concurrency-runtime-stdlib-parity.md).
- This phase is the first phase in the split production-stdlib sequence because it provides the shared encoding/text substrate needed by both the network/HTTP and concurrency/runtime phases.
- This phase provides the encoding/text substrate needed by subprocess text mode, HTTP text decoding, file `open(..., encoding=...)`, warning formatting, locale-aware formatting, and translation demos.
- This phase assumes [ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md](./ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md) is complete: Sifr stdlib remains publicly imported through `sifr.*`, and bare CPython stdlib names are not aliases.

## Cross-Phase Dependency Contract

The split phase order is explicit: this text/Unicode/encoding/i18n phase is implemented first, then the network/HTTP and concurrency/runtime phases can implement their text-dependent surfaces on top of it.

- Network/web may implement only binary socket/TLS/HTTP and byte/ASCII/UTF-8 URL behavior before this phase's text gates are complete. Non-UTF-8 HTTP body decoding, file/text handlers requiring encoding lookup, and text-heavy demos are blocked until `milestone_text_i18n_1: Encoding And Explicit Text I/O` is complete.
- Concurrency/runtime may implement only binary subprocess pipes before this phase's text gates are complete. `text=True`, `encoding=...`, `errors=...`, warning output encoding, and demos that require text-mode `open` are blocked until `milestone_text_i18n_1` is complete; locale-sensitive warning formatting still also waits for `milestone_text_i18n_3`.
- Binary-mode file I/O is prior stdlib/runtime infrastructure owned by the earlier runtime/file-object parity work (`issues/archive/ad-hoc-runtime-and-file-object-parity-expansion.md`) and the current `sifr.io` surface. M0 must verify that binary `open()`/file handles are usable before text-mode integration starts.
- Before M0 starts, run a binary file I/O smoke check covering binary `open`, read, write, seek/tell where supported, close/drop, error-on-use-after-close, and byte-preserving round trips. If it fails, M0 may record inventory only; `milestone_text_i18n_1` is blocked until the stdlib/runtime file-object owner fixes `sifr.io` in a prerequisite PR recorded in this phase's execution ledger. Text-mode code must not work around or duplicate broken binary I/O behavior.
- This phase owns only text-mode integration through explicit text-reader/text-writer APIs and `open(..., encoding=..., errors=...)` lowering.
- Text-mode `open(path)`, `open(path, mode="r")`, and any other text mode without an explicit `encoding=` are unsupported in this phase and remain unsupported after M3. Sifr requires explicit text encodings to preserve static behavior; M3 documents this as an intentional difference from CPython's locale-derived default.
- Text stream wrappers such as `io.TextIOWrapper(binary_stream, ...)` follow the same policy: explicit `encoding=` is required for text wrapping, and omitted/default locale-derived encoding is unsupported with diagnostics. If `io.TextIOWrapper` itself is not implemented in this phase, that surface is recorded as `unsupported` with CPython evidence.
- In-memory text streams such as `io.StringIO` are encoding-free because they operate on native strings. If touched by this phase, `StringIO` must not require an `encoding=` argument and its `newline=` parameter is either implemented with CPython-compatible universal-newline behavior or rejected with an unsupported-parameter diagnostic. Otherwise it remains existing `sifr.io` prior infrastructure or is recorded as unsupported separately from codec-backed wrappers.
- `open(...)` mode strings must be statically known for the compiler to choose binary versus text handle types. Nonliteral/dynamic mode strings are unsupported unless routed through a future typed helper API; this avoids false-positive encoding diagnostics for dynamic binary modes and avoids a handle type that depends on runtime string contents.
- Statically visible text-mode opens without `encoding=` produce a compile-time diagnostic requiring `encoding=...`. Dynamic/nonliteral mode opens produce a compile-time diagnostic requiring a literal mode or typed helper. Sifr must not silently substitute UTF-8 for CPython's locale-derived default.
- Consumers must call the encoding/text substrate owned by this phase; they must not add local encoding fallbacks.

## Source Of Truth

The authoritative CPython source tree for reference scanning is:

- `/Users/yaseralnajjar/work/sifr/cpython`

The implementation must scan and classify these CPython files before each milestone implementation PR, but the scan classifies surfaces as production substrate, future adapter, host-limited, rejected, or deferred. It does not automatically promote Python-shaped APIs into Sifr production APIs.

| Domain | CPython library sources | CPython test sources | Native backing sources |
| --- | --- | --- | --- |
| codecs/encodings | `Lib/codecs.py`, `Lib/encodings/*.py`, `Doc/library/codecs.rst` | `Lib/test/test_codecs.py`, `Lib/test/test_capi/test_codecs.py` | `Modules/_codecsmodule.c`, `Modules/cjkcodecs/*` |
| Unicode data | `Doc/library/unicodedata.rst` | `Lib/test/test_unicodedata.py` | `Modules/unicodedata.c` |
| locale/gettext | `Lib/locale.py`, `Lib/gettext.py`, `Doc/library/locale.rst`, `Doc/library/gettext.rst` | `Lib/test/test_locale.py`, `Lib/test/test_gettext.py` | `Modules/_localemodule.c` |

Path note: CPython paths above are relative to `/Users/yaseralnajjar/work/sifr/cpython`.

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

Every reviewed CPython test family must end in exactly one state: `adopted-as-substrate-fixture`, `adapted-for-sifr-api`, `adapter-deferred`, `waived`, or `rejected`. Every reviewed public surface must end in exactly one state: `done`, `intentional-diff`, `unsupported`, `host-limited`, or `deferred-adapter`. The inventory state `open` is forbidden at phase exit.

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
  - sentence boundaries only if M2.5 accepts the dependency/data cost
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

If any Python-shaped module is implemented later, it must be a thin adapter over the Sifr-native substrate, recorded as `deferred-adapter`, and reviewed for static typing, ownership, no-global-state, and panic-free behavior. Bare CPython module imports such as `import codecs`, `import locale`, or `import gettext` are not aliases for `sifr.*`; they receive the namespace-contract diagnostic after normal user/package resolution fails.

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
- Registry mutation is not adopted in this phase. Dynamic codec lookup by name is supported only against the static registry.
- `codecs.register`, `codecs.unregister`, `codecs.register_error`, dynamic error-handler lookup, `codecs.open`, `EncodedFile`, `StreamReader`, `StreamWriter`, `StreamReaderWriter`, and full `CodecInfo` compatibility are not required phase outputs.

### Encoding Support Tiers

Encoding support is demand-tiered:

| Tier | Required status | Encodings |
| --- | --- | --- |
| Tier 0 | Required for phase exit | `utf-8`, `utf-8-sig`, `ascii`, exact `latin-1`, `utf-16-le`, `utf-16-be`; `utf-32-le`/`utf-32-be` only if table/runtime cost is acceptable |
| Tier 1 | Required if dependency and binary-size review pass | `windows-1252`, selected `windows-125x`, WHATWG label decoding for HTTP/HTML boundaries |
| Tier 2 | Feature-gated and workload-justified | `Shift_JIS`, `EUC-JP`, `GBK`, `GB18030`, `Big5`, `EUC-KR` |
| Tier 3 | Deferred/rejected for this phase | obscure CPython-only aliases, text-to-text codecs, bytes-to-bytes pseudo-codecs, full public `encodings.*` module parity |

M0 must record the exact Tier 0/Tier 1 selection and any binary-size, license, or generated-table constraints before M1 starts.

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

- M0 must record the Unicode data version and the CPython checkout version used for reference fixtures.
- Generated Unicode tables must be reproducible, reviewed, and excluded from the hand-maintained 900-line guardrail where appropriate.
- Runtime APIs must expose the Unicode data version consistently with the table set used by the build.
- Normalization, properties, case folding, and segmentation must either share the same Unicode version or explicitly record any version skew as a release blocker.

### Locale And I18n Without Process Globals

- Sifr does not clone Python's process-global `locale` model as a production API.
- `setlocale`-style mutation, `localeconv` as a primary API, `strcoll`, `strxfrm`, `format_string`, and `currency` are unsupported or deferred adapters unless a later phase proves they can be implemented without global mutation hazards.
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
- `CatalogParseError`

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
- CJK/all-codepage parity without feature gating, dependency review, and workload justification
- host-specific locale names that cannot be made deterministic on the supported host matrix

## Milestones

### milestone_text_i18n_0: Product Boundary And Rust Lowering Contract

Scope:

- Add a machine-readable substrate inventory under `verification/stdlib/text_i18n_substrate_inventory.*`.
- Scan every source/test/doc file listed in `Source Of Truth`.
- Extract public functions, classes, constants, methods, common keyword forms, current codec aliases, encoding module names, deprecation/legacy markers, and test-class/test-method names.
- Classify each extracted CPython surface as `production-substrate`, `future-adapter`, `host-limited`, `rejected`, or `deferred`.
- Add Sifr-native fixture plans for:
  - `sifr_encoding_subset.sifr`
  - `sifr_text_io_subset.sifr`
  - `sifr_unicode_subset.sifr`
  - `sifr_unicode_segmentation_subset.sifr`
  - `sifr_i18n_locale_formatting_subset.sifr`
  - `sifr_i18n_translation_subset.sifr`
- Add negative import-resolution tests for bare CPython stdlib import attempts and avoid positive tests for Python-shaped modules unless a later adapter phase accepts them.
- Define Text versus Bytes invariants and the Rust lowering contract.
- Decide:
  - public API names under `sifr.encoding`, `sifr.unicode`, `sifr.io`, and `sifr.i18n`
  - Unicode data version
  - generated table strategy
  - supported encoding tiers
  - selected Rust crates and feature flags
  - typed encode/decode error handlers
  - recovery-carrying value shape
  - surrogate behavior policy
  - host-limited locale discovery matrix
  - diagnostic wording for explicit-encoding-required text `open(...)` and literal-mode-required `open(...)`
  - translation backend scope and `.mo` support decision
- Assign each inventory entry one owner milestone and one terminal state.

Definition of done:

- The backlog is derived from CPython sources/tests, Unicode/encoding/i18n standards, and selected Rust runtime crates rather than hand-written memory.
- Every target capability has a first-pass surface matrix and fixture matrix.
- Every CPython-shaped surface has a disposition and is not accidentally treated as required production API.
- M1-M5 implementation PRs, including M2.5, have concrete backlog entries rather than prose-only scope.

### milestone_text_i18n_1: Encoding And Explicit Text I/O

Scope:

- Provide the exact cross-phase unblock point for:
  - network/web `blocked-on-text-i18n-m1` surfaces: non-UTF-8 URL quoting/parsing forms, HTTP body text decoding, and network demos that require `open(..., encoding=...)`
  - concurrency/runtime `blocked-on-text-i18n-m1` surfaces: subprocess `text=True`, `encoding=...`, `errors=...`, warning output encoding, and demos that require text-mode `open`
  - locale-sensitive warning formatting remains additionally blocked on `milestone_text_i18n_3`
- Add `sifr.encoding`:
  - `Encoding`
  - static encoding labels and aliases accepted by M0
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
- Feature-gate any accepted Tier 2 encodings.
- Add explicit text I/O integration:
  - `sifr.io.open_text(path, encoding=..., errors=...)`
  - `TextReader`
  - `TextWriter`
  - `open(..., encoding=..., errors=...)` lowering
  - `io.TextIOWrapper(..., encoding=..., errors=...)` only if adopted; otherwise record as unsupported
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
  - simple/full case mapping only as accepted by M0's data-size review
- Generate or vendor Unicode data tables according to the M0 version decision.
- Ensure normalization, property queries, and case mapping share the same table version.

CPython tests to mine for fixture ideas and waiver evidence:

- `Lib/test/test_unicodedata.py`

Rust/runtime candidates:

- `unicode-normalization`
- generated Unicode tables
- ICU4X components if selected by M0

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
  - sentence boundaries only if accepted by M0/M2.5 dependency and data review
- Define iterator ownership and allocation behavior.
- Define streaming segmentation only if needed for file/network workloads; otherwise record it as future work.
- Ensure segmentation data version aligns with M2 or record version skew as a release blocker.

Rust/runtime candidates:

- `unicode-segmentation`
- ICU4X segmentation components if selected by M0

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
  - likely-subtag behavior only if dependency/data cost is accepted
- Add object-based formatting:
  - `NumberFormatter`
  - `DateTimeFormatter`
  - `PluralRules`
  - `Collator` where dependency/data cost is acceptable
  - display names only if explicitly accepted by M0
- Add read-only host locale discovery only if needed by runtime tooling or demos, and mark it `host-limited`.
- Do not add process-global `setlocale` behavior.
- Do not make locale preferred encoding APIs legalize text `open(...)` without `encoding=`.

CPython tests to mine for anti-requirements, host-limited evidence, and fixture ideas:

- `Lib/test/test_locale.py`

Rust/runtime candidates:

- ICU4X/ICU-style components
- `unic-langid` or successor locale identifier crates if accepted by M0

Definition of done:

- Locale identifiers are parsed and canonicalized deterministically.
- Number/date formatting, plural rules, and accepted collation behavior use explicit locale values and formatter objects.
- Supported locale data and host assumptions are documented and tested.
- Python `locale` process-global APIs are recorded as unsupported/deferred adapters, not production surfaces.
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
- Add `.mo` loader/importer if M0 accepts gettext as a cheap compatibility backend.
- Integrate `.mo` decoding with the M1 encoding substrate.
- Add future Fluent/ICU message backend hook if it can be designed without committing to the backend in this phase.
- Do not add `gettext.install` or global `_` mutation.

CPython tests to mine for fixture ideas and waiver evidence:

- `Lib/test/test_gettext.py`

Rust/runtime candidates:

- `.mo` parser implemented in Sifr runtime/Rust
- `fluent` only if M0 accepts it as a future-facing backend or optional feature

Definition of done:

- Translation parsing is deterministic and panic-free.
- Plural forms, contexts, fallback chains, missing keys, and missing catalog paths have Sifr-native fixtures.
- `.mo` support, if accepted, is documented as an import/backend format rather than the strategic i18n API.
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

- Every production API and reference test family in the phase inventory is closed as `done`, `intentional-diff`, `unsupported`, `host-limited`, or `deferred-adapter`.
- No implementation-owned source file exceeds the 900-line guardrail.
- No user-triggerable runtime panic path exists in the added stdlib/runtime surfaces.
- No unsynchronized process-global mutation exists.
- CPython-shaped adapter work is either explicitly deferred or separately accepted with Sifr-native substrate backing.

## Required Tracking Artifacts

Create and keep current during implementation:

- `issues/ad-hoc-production-text-i18n-stdlib-parity-execution.md`
- `verification/stdlib/text_i18n_substrate_inventory.md`
- `verification/stdlib/text_i18n_substrate_inventory.json`
- `verification/stdlib/text_i18n_reference_matrix.md`
- one traceability document per milestone domain under `verification/stdlib/`

The execution ledger must record:

- planning/review artifacts
- per-milestone PR links
- local validation commands and results
- CPython source/test files scanned
- standards and Rust crates reviewed
- adopted/adapted/waived/deferred/rejected reference test families
- final unsupported/intentional-diff/host-limited/deferred-adapter index

## Quality Contract

- Solve root causes rather than adding workaround wrappers.
- No backward-compatibility shims, legacy aliases, deprecated behavior, implicit locale-default behavior, fallback paths, or CPython-shaped public API commitments may survive phase exit unless separately accepted as adapters over the Sifr-native substrate.
- No unsynchronized process-global state may be introduced.
- No data-dependent emitted `.unwrap()`, `.expect()`, or `panic!` is allowed in user runtime paths.
- Every added CPU-heavy or blocking sync function must be classified in the stdlib workload database.
- Every added external crate dependency must be represented by a stable `StdlibFeature` in `sifr_stdlib`.
- Every production module added to embedded stdlib sources must have canonical `sifr.*` import-resolution tests, type-check tests, e2e pass tests, and negative diagnostics for unsupported bare CPython import forms.

## M0 Implementation Decisions To Record

1. What are the exact public API names and type shapes for `sifr.encoding`, `sifr.unicode`, `sifr.io`, and `sifr.i18n`?
2. Which encoding families are required for phase exit, feature-gated, or explicitly deferred?
3. Which Unicode data version is shipped, and how are generated tables produced reproducibly?
4. Which static encoding aliases are shipped, and what diagnostics are used for unsupported registry mutation?
5. Which typed error handlers are accepted for encode and decode paths?
6. How are recovery diagnostics represented without hiding invalid Unicode inside normal strings?
7. Which surrogate-preserving behavior is rejected, deferred, or isolated behind byte-preserving boundary types?
8. Which crate/data strategy meets binary-size, license, safety, and maintenance goals for encodings, Unicode data, segmentation, locale formatting, plural rules, and collation?
9. Which host locale queries are supported in local validation, and how are host-limited outcomes represented?
10. Is `.mo` support cheap enough for this phase, and is any Fluent/ICU message backend hook accepted?
11. What exact diagnostics and waiver evidence are recorded for unsupported Python-shaped APIs such as `setlocale`, `codecs.register`, `codecs.register_error`, `gettext.install`, and full `encodings.*` parity?

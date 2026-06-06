# Ad Hoc Phase: Production Text And Internationalization Stdlib Parity

Status: draft
Phase placement: ad hoc expansion phase after the stdlib boundary refactor and before any stable GA claim that Sifr is production-ready for internationalized programs or non-UTF-8 text I/O.
Phase owner: stdlib/runtime implementation with compiler import, file/text I/O, effect, and codegen support

## Objective

Close the production stdlib gaps for text encoding, Unicode data, locale, and translation support:

- codec registry and encodings: `codecs`, `encodings`
- Unicode properties and normalization: `unicodedata`
- locale-aware text/numeric behavior: `locale`
- message catalogs and translation: `gettext`

This phase is complete when each target surface has either:

- current-CPython-shaped source parity with Sifr-safe semantics,
- a native Sifr text/i18n runtime implementation that backs that compatibility surface,
- or an explicit, tested waiver with rationale, revisit rule, and CPython test-family evidence.

This phase does not add backward-compatibility or legacy support. Parity means the current supported CPython stdlib API shape and behavior adapted under Sifr's canonical `sifr.*` namespace with Sifr's static, typed, ownership-safe model. Bare CPython stdlib imports, historical aliases, deprecated APIs, implicit locale-default text behavior, compatibility shims, and hidden bridge names are not implemented; they receive diagnostics or waivers.

## Related Phases

- Network/HTTP platform substrate remains in [ad-hoc-production-network-http-platform-substrate.md](./ad-hoc-production-network-http-platform-substrate.md).
- Concurrency/runtime stdlib parity is tracked in [ad-hoc-production-concurrency-runtime-stdlib-parity.md](./ad-hoc-production-concurrency-runtime-stdlib-parity.md).
- This phase provides the codec/text substrate needed by subprocess text mode, HTTP text decoding, file `open(..., encoding=...)`, warning formatting, locale-aware formatting, and gettext demos.
- This phase assumes [ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md](./ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md) is complete: Sifr stdlib remains publicly imported through `sifr.*`, and bare CPython stdlib names are not aliases.

## Cross-Phase Dependency Contract

The three split phases are not an implied ship order. This phase is the provider for text-dependent features in the other two phases:

- Network/web may ship binary socket/TLS/HTTP and byte/ASCII/UTF-8 URL behavior before this phase, but non-UTF-8 HTTP body decoding, file/text handlers requiring codec lookup, and text-heavy demos are unblocked only after `milestone_text_i18n_1: Codecs Registry, Encodings, And Text I/O Integration` is complete.
- Concurrency/runtime may ship binary subprocess pipes before this phase, but `text=True`, `encoding=...`, `errors=...`, warning output encoding, and locale-sensitive warning formatting are unblocked only after `milestone_text_i18n_1` is complete; any locale-specific formatting still also waits for `milestone_text_i18n_3`.
- Binary-mode file I/O is prior stdlib/runtime infrastructure owned by the earlier runtime/file-object parity work (`issues/archive/ad-hoc-runtime-and-file-object-parity-expansion.md`) and the current `sifr.io` surface. M0 must verify that binary `open()`/file handles are usable before text-mode integration starts.
- Before M0 starts, run a binary file I/O smoke check covering binary `open`, read, write, seek/tell where supported, close/drop, error-on-use-after-close, and byte-preserving round trips. If it fails, M0 may record inventory only; `milestone_text_i18n_1` is blocked until the stdlib/runtime file-object owner fixes `sifr.io` in a prerequisite PR recorded in this phase's execution ledger. Text-mode code must not work around or duplicate broken binary I/O behavior.
- This phase owns only text-mode integration through `open(..., encoding=..., errors=...)`.
- Text-mode `open(path)`, `open(path, mode="r")`, and any other text mode without an explicit `encoding=` are unsupported in this phase and remain unsupported after M3. Sifr requires explicit text encodings to preserve static behavior; M3 documents this as an intentional difference from CPython's locale-derived default.
- Text stream wrappers such as `io.TextIOWrapper(binary_stream, ...)` follow the same policy: explicit `encoding=` is required for text wrapping, and omitted/default locale-derived encoding is unsupported with diagnostics. If `io.TextIOWrapper` itself is not implemented in this phase, that surface is recorded as `unsupported` with CPython evidence.
- In-memory text streams such as `io.StringIO` are encoding-free because they operate on native strings. If touched by this phase, `StringIO` must not require an `encoding=` argument and its `newline=` parameter is either implemented with CPython-compatible universal-newline behavior or rejected with an unsupported-parameter diagnostic. Otherwise it remains existing `sifr.io` prior infrastructure or is recorded as unsupported separately from codec-backed wrappers.
- `open(...)` mode strings must be statically known for the compiler to choose binary versus text handle types. Nonliteral/dynamic mode strings are unsupported unless routed through a future typed helper API; this avoids false-positive encoding diagnostics for dynamic binary modes and avoids a handle type that depends on runtime string contents.
- Statically visible text-mode opens without `encoding=` produce a compile-time diagnostic requiring `encoding=...`. Dynamic/nonliteral mode opens produce a compile-time diagnostic requiring a literal mode or typed helper. Sifr must not silently substitute UTF-8 for CPython's locale-derived default.
- Consumers must call the codec registry owned by this phase; they must not add local encoding fallbacks.

## Source Of Truth

The authoritative CPython source tree for this phase is:

- `/Users/yaseralnajjar/work/sifr/cpython`

The implementation must scan and classify these CPython files before each milestone implementation PR:

| Domain | CPython library sources | CPython test sources | Native backing sources |
| --- | --- | --- | --- |
| codecs/encodings | `Lib/codecs.py`, `Lib/encodings/*.py`, `Doc/library/codecs.rst` | `Lib/test/test_codecs.py`, `Lib/test/test_capi/test_codecs.py` | `Modules/_codecsmodule.c`, `Modules/cjkcodecs/*` |
| Unicode data | `Doc/library/unicodedata.rst` | `Lib/test/test_unicodedata.py` | `Modules/unicodedata.c` |
| locale/gettext | `Lib/locale.py`, `Lib/gettext.py`, `Doc/library/locale.rst`, `Doc/library/gettext.rst` | `Lib/test/test_locale.py`, `Lib/test/test_gettext.py` | `Modules/_localemodule.c` |

Path note: CPython paths above are relative to `/Users/yaseralnajjar/work/sifr/cpython`.

## Current Sifr Baseline

- `sifr.io` has file handles and in-memory stream wrappers, but no codec-driven text stream layer beyond current UTF-8-oriented boundaries.
- Binary-mode file I/O is existing `sifr.io` infrastructure and remains outside this phase except where text-mode wrappers compose over it.
- `str.encode(...)`, `bytes.decode(...)`, and `open(..., encoding=...)` do not have production CPython-style codec registry parity.
- `sifr.codecs`, `sifr.encodings`, `sifr.unicodedata`, `sifr.locale`, and `sifr.gettext` are not present as production stdlib surfaces.

The Phase 32 async/workload model remains binding:

- CPU-heavy table generation, normalization, or large codec work must be classified as `@cpu_heavy` where appropriate.
- Blocking file reads for `.mo` catalogs or locale resources must be classified as `@blocking_io`.
- Direct calls to blocking or CPU-heavy sync APIs from `async def` remain compiler errors unless routed through native async APIs or explicit offload.

## Parity Definition

This phase targets current CPython-shaped interfaces under the canonical `sifr.*` namespace, not legacy compatibility layers or bare CPython import compatibility.

For each module in scope:

1. Support canonical Sifr stdlib imports for the CPython-shaped surface (`from sifr import codecs`, `from sifr import unicodedata`, `from sifr import locale`, etc.).
2. Do not add bare CPython module-name imports as aliases for `sifr.*`. Bare forms such as `import codecs` or `import locale` should receive the namespace-contract diagnostic once normal user/package resolution fails.
3. Match CPython function/class names, constructor forms, constants, and common keyword arguments where compatible with Sifr's static type system.
4. Adapt CPython exception behavior into Sifr-safe `Result[T, E]`, `Option[T]`, or compile-time diagnostics.
5. Keep host-specific behavior explicitly marked `host-limited`.
6. Keep CPython implementation-detail, deprecated, and historical compatibility behavior waived rather than reimplemented blindly.

Every reviewed CPython test family must end in exactly one state: `adopted`, `adapted`, or `waived`. Every public surface must end in exactly one state: `done`, `intentional-diff`, `unsupported`, or `host-limited`. `open` is forbidden at phase exit.

## Milestone Dependency Graph

1. `milestone_text_i18n_0` first. No implementation milestone starts until the inventory, CPython test matrix, import plan, shared error mapping, Unicode version decision, and codec registry mutation policy are checked in.
2. `milestone_text_i18n_1` before all consumers. Codec registry basics and core encodings are the substrate for file I/O, subprocess text mode, HTTP text decoding, and gettext catalog parsing.
3. `milestone_text_i18n_2` can run after M0 but must lock the Unicode data version before normalization/property APIs ship.
4. `milestone_text_i18n_3` waits for M1 where locale APIs depend on encodings.
5. `milestone_text_i18n_4` waits for M1 for `.mo` file decoding and declared catalog encodings.
6. `milestone_text_i18n_5` closes docs, demos, validation, and waivers last.

## Architecture Principles

### Native Registry First, Compatibility Second

Implement the canonical codec/text primitive first, then layer CPython-shaped modules over it.

- A private runtime registry owns codec lookup, encoder/decoder construction, and error handler lookup.
- CPython-shaped canonical Sifr modules (`sifr.codecs`, `sifr.encodings.*`) delegate to that registry.
- `str.encode(...)`, `bytes.decode(...)`, and `open(..., encoding=...)` must use the same registry to avoid dual semantics.
- Registry mutation is not adopted in this phase. The phase ships a static built-in registry, records `codecs.register`/`codecs.unregister` as `unsupported`/`intentional-diff` with CPython evidence, and may revisit synchronized mutation only in a future registry-mutation phase. Dynamic codec lookup by name is still supported against the static registry.

### Incremental Codec Ownership

`IncrementalEncoder` and `IncrementalDecoder` are stateful linear values:

- encode/decode calls require a unique mutable handle (`&mut`-equivalent in the lowered Rust model)
- the compiler rejects concurrent aliasing of the same incremental codec object
- incremental codec objects are not `Send`/`Sync` and are not shareable across tasks/threads unless a future explicit locked wrapper API is added
- `final=True` transitions the object to an exhausted state; later calls through the same handle return typed exhausted errors
- CPython-shaped wrapper objects lower to this same unique-mutable state model rather than `RefCell`/hidden global state

### Text Data Versioning

- M0 must record the Unicode data version and the CPython checkout version used for parity fixtures.
- Generated Unicode tables must be reproducible, reviewed, and excluded from the hand-maintained 900-line guardrail where appropriate.
- Runtime APIs must expose `unicodedata.unidata_version` consistently with the table set used by the build.

### Typed Errors Instead Of Exceptions

All fallible APIs must expose typed error results:

- `CodecError`, `UnicodeEncodeError`, `UnicodeDecodeError`, `UnicodeTranslateError`
- `UnicodeDataError`
- `LocaleError`
- `GettextError`

Names may align with CPython where possible, but the operational contract is Sifr `Result`/`Option`, not exception-driven control flow.

### Panic-Free Runtime Contract

Generated Rust for these APIs must not contain data-dependent `.unwrap()`, `.expect()`, or `panic!` on user-controlled text, byte sequences, locale names, format strings, or `.mo` catalog data.

### Global-State Discipline

- Codec registry mutation, locale mutation, and gettext global installation never create unsynchronized process-global state.
- Locale-mutating APIs adopted in this phase use a process-global lock around `setlocale`, `localeconv`, `strcoll`, `strxfrm`, and formatting operations that consult mutable locale state. Locale names and platform-specific behavior remain `host-limited` where the supported host matrix cannot make them deterministic.
- Locale state is process-scoped. Threaded and process-pool code from the concurrency/runtime phase must either serialize locale-sensitive operations through this phase's locale lock or record host-limited/intentional-diff behavior.
- `gettext.install`-style global mutation is waived/unsupported in this phase. Explicit `translation(...)` objects, fallback chains, and direct `gettext`/`ngettext` calls are the supported path; global builtins/module mutation gets diagnostics plus CPython evidence and a revisit rule.

## Non-Goals And Permanent Boundaries

The following are not accepted as silent omissions. They must be either implemented or explicitly waived with tests:

- codec registry mutation through `codecs.register`/`codecs.unregister`; unsupported in this phase
- unsynchronized process-global locale mutation
- dynamic monkeypatching of module globals
- C API compatibility for external CPython extensions
- CJK encodings before dependency size, generated table size, and test cost are reviewed
- host-specific locale names that cannot be made deterministic on the supported host matrix

## Milestones

### milestone_text_i18n_0: CPython Inventory, Error Mapping, And Registry Design

Scope:

- Add a machine-readable parity inventory under `verification/stdlib/text_i18n_parity_inventory.*`.
- Scan every source/test/doc file listed in `Source Of Truth`.
- Extract public functions, classes, constants, methods, common keyword forms, current codec aliases, encoding module names, deprecation/legacy markers, and test-class/test-method names. Legacy-only codec aliases are waived.
- Add CPython-derived e2e fixtures:
  - `cpython_codecs_subset.sifr`
  - `cpython_encodings_subset.sifr`
  - `cpython_unicodedata_subset.sifr`
  - `cpython_locale_subset.sifr`
  - `cpython_gettext_subset.sifr`
- Add import-resolution tests for canonical `sifr.*` module names and negative diagnostics for bare CPython stdlib import attempts.
- Add shared error mapping for all text/i18n target domains.
- Decide:
  - Unicode data version
  - generated table strategy
  - static registry alias table and unsupported diagnostics for registry mutation
  - locale global-state lock implementation and host-limited locale matrix
  - diagnostic wording for explicit-encoding-required text `open(...)` and literal-mode-required `open(...)`
  - gettext global installation unsupported diagnostics and waiver evidence
- Assign each inventory entry one owner milestone and one terminal state.
- Assign every deprecated, historical, or legacy-only entry the terminal state `unsupported` or `intentional-diff`. M0 may implement only current, non-deprecated target CPython surfaces that remain elegant under Sifr semantics.

Definition of done:

- The backlog is derived from CPython source/tests, not hand-written memory.
- Every target module has a first-pass surface matrix and CPython test-family matrix.
- M1-M5 implementation PRs have concrete backlog entries rather than prose-only scope.

### milestone_text_i18n_1: Codecs Registry, Encodings, And Text I/O Integration

Scope:

- Provide the exact cross-phase unblock point for:
  - network/web `blocked-on-text-i18n-m1` surfaces: non-UTF-8 URL quoting/parsing forms, HTTP body text decoding, and network demos that require `open(..., encoding=...)`
  - concurrency/runtime `blocked-on-text-i18n-m1` surfaces: subprocess `text=True`, `encoding=...`, `errors=...`, warning output encoding, and demos that require text-mode `open`
  - locale-sensitive warning formatting remains additionally blocked on `milestone_text_i18n_3`

- Add `codecs` registry:
  - `lookup`
  - `register`, `unregister` are unsupported/intentional-diff in this phase because the registry is static
  - `encode`, `decode`
  - `getencoder`, `getdecoder`
  - `getincrementalencoder`, `getincrementaldecoder`
  - `iterencode`, `iterdecode`
  - `CodecInfo`
  - `IncrementalEncoder`, `IncrementalDecoder`
  - incremental codec finalization uses explicit exhausted state: `encode(..., final=True)` and `decode(..., final=True)` flush pending state and mark the encoder/decoder exhausted; subsequent encode/decode calls return typed `CodecError::EncoderExhausted`/`CodecError::DecoderExhausted` or equivalent rather than silently succeeding or panicking
  - implementations may add consuming/type-state overloads for statically known `final=True`, but the runtime contract must still handle dynamic `final` values with typed exhaustion errors
  - error handlers: `strict`, `ignore`, `replace`, `backslashreplace`, `namereplace`, `xmlcharrefreplace`, `surrogateescape`, `surrogatepass` where compatible with Sifr string invariants
  - `errors=` parameters for `str.encode`, `bytes.decode`, `open`, and text wrappers use a typed error-handler enum or statically known string literals mapped to that enum
  - M1 must define an explicit error-handler applicability table with at least:
    - encode-only: `xmlcharrefreplace`, `namereplace`
    - bidirectional: `strict`, `ignore`, `replace`, `backslashreplace`
    - codec-limited bidirectional: `surrogateescape`, `surrogatepass` only where compatible with the selected Unicode/codec invariants
  - encode and decode APIs use separate typed handler parameters, such as `EncodeErrorHandler` and `DecodeErrorHandler`, so encode-only handlers are rejected from decode call sites by signature when statically known
  - statically known string literals are lowered to those typed handler values; invalid literal/context combinations produce compile-time diagnostics
  - dynamic handler names are unsupported in this phase because synchronized runtime error-handler lookup is not adopted; invalid context combinations for static literals produce compile-time diagnostics
  - codec-limited handlers are rejected outside their valid codec/context; bidirectional handlers such as `backslashreplace` must remain valid for both encode and decode
  - dynamic `errors=` strings are unsupported in this phase because synchronized runtime error-handler lookup is not adopted; no silent fallback to `strict`
  - strict incremental encode/decode failures return `Err(CodecError::...)` with no successful partial-output value
  - recoverable non-strict handlers (`ignore`, `replace`, `backslashreplace`, and codec-limited handlers where valid) return typed success outcomes that preserve both produced output and recovery evidence, such as `DecodeOutcome { text, recoveries }` or `EncodeOutcome { bytes, recoveries }`
  - CPython-shaped convenience wrappers may expose only the produced text/bytes where CPython does, but the internal runtime contract must retain recovery diagnostics for validation, tracing, and typed adapter layers; recovery must not be silently discarded in lower-level runtime APIs
- Add encoding families in waves:
  - UTF core: `utf-8`, `utf-8-sig`, `utf-16`, `utf-16-le`, `utf-16-be`, `utf-32`, `utf-32-le`, `utf-32-be`
  - Latin/ASCII: `ascii`, `latin-1`, ISO-8859 family
  - URL/web adjacent: `idna`, `punycode`
  - common Windows/code pages: `cp1250`-`cp1258`, selected `cp437`/`cp850`
  - CJK encodings only after dependency and test size are reviewed
- Integrate:
  - `str.encode(...)`
  - `bytes.decode(...)`
  - `open(..., encoding=..., errors=...)`
  - `io.TextIOWrapper(..., encoding=..., errors=...)` if text stream wrappers are adopted; otherwise record `io.TextIOWrapper` as unsupported
  - no-encoding text `open(...)` is not implemented; it emits the explicit-encoding-required diagnostics defined by M0
  - subprocess text mode and HTTP decoding only after their owning phases consume the registry

CPython tests to mine:

- `Lib/test/test_codecs.py`
- `Lib/test/test_capi/test_codecs.py`

Rust/runtime candidates:

- `encoding_rs`
- generated tables for encodings not covered by selected crates

Definition of done:

- Core codec lookup, aliases, BOM handling, incremental encoding/decoding, and error handlers pass CPython-derived fixtures.
- Incremental encoder/decoder finalization and post-finalization exhaustion have fixtures for both statically known and dynamic `final` values.
- Incremental codec fixtures cover mid-stream strict errors with no partial success and recoverable non-strict errors that return partial output plus recovery diagnostics.
- `str.encode(encoding, errors)` and `bytes.decode(encoding, errors)` have fixtures for supported error-handler literals, unsupported dynamic handler names, and typed error-handler values.
- Encode/decode context restrictions for error handlers have fixtures, including rejecting encode-only handlers on decode call sites.
- Text I/O uses the same codec registry as explicit encode/decode APIs.
- Registry mutation is synchronized or explicitly waived with tests.
- Static-registry behavior has fixtures for lookup, alias resolution, unsupported `register`/`unregister`, and no silent fallback on missing codecs.

### milestone_text_i18n_2: Unicode Data And Normalization

Scope:

- Add `unicodedata`:
  - `name`
  - `lookup`
  - `category`
  - `bidirectional`
  - `combining`
  - `east_asian_width`
  - `mirrored`
  - `decomposition`
  - `normalize`
  - `is_normalized`
  - `decimal`, `digit`, `numeric`
  - `unidata_version`
- Generate or vendor Unicode data tables according to the M0 version decision.
- Ensure normalization and property queries share the same table version.

CPython tests to mine:

- `Lib/test/test_unicodedata.py`

Rust/runtime candidates:

- `unicode-normalization`
- `unicode-general-category`
- generated Unicode tables if crate coverage is insufficient

Definition of done:

- Unicode normalization and property queries pass CPython-derived fixtures.
- `unidata_version` matches the shipped table data.
- Missing-name and missing-property paths return typed errors/options, never panics.

### milestone_text_i18n_3: Locale

Scope:

- Add `locale`:
  - `getlocale`
  - `getdefaultlocale` is waived/unsupported as deprecated; no deprecated locale-default behavior is implemented for backward compatibility
  - `getpreferredencoding`
  - `getencoding`
  - `setlocale`, `localeconv`, `strcoll`, `strxfrm`
  - `format_string`, `currency`, `atof`, `atoi`, `normalize`
  - process-global mutation rules and thread-safety contract
  - read-only locale queries (`getpreferredencoding`, `getencoding`, `localeconv` snapshots where possible) are prioritized before mutating APIs
  - `setlocale`/`strcoll`/`strxfrm`/formatting APIs must be guarded by a process-global lock or marked `host-limited`/`intentional-diff` if deterministic concurrent use cannot be guaranteed
  - locale preferred encoding APIs do not make text `open(...)` without `encoding=` legal; explicit encoding remains required

CPython tests to mine:

- `Lib/test/test_locale.py`

Rust/runtime candidates:

- platform locale APIs behind synchronized wrappers
- `unic-langid` only if locale parsing needs it

Definition of done:

- Locale process-global behavior is synchronized and host-limited where necessary.
- Supported locale names and host assumptions are documented and tested.
- Locale errors are typed and never panic.

### milestone_text_i18n_4: Gettext

Scope:

- Add `gettext`:
  - `NullTranslations`
  - `GNUTranslations`
  - `translation`
  - `find`
  - `gettext`, `ngettext`, `pgettext`, `npgettext`
  - `.mo` file parsing with typed errors
  - global installation only if synchronized and explicit; otherwise waived
- Integrate `.mo` decoding with the M1 codec registry.

CPython tests to mine:

- `Lib/test/test_gettext.py`

Definition of done:

- Gettext `.mo` parsing is deterministic and panic-free.
- Plural forms, contexts, fallback chains, and missing catalog paths have CPython-derived fixtures.
- Global installation is synchronized or explicitly waived.

### milestone_text_i18n_5: Integration, Documentation, And Production Gate

Scope:

- Update public docs for every new module and major intentional divergence:
  - `codecs`, `encodings`
  - `unicodedata`
  - `locale`
  - `gettext`
- Update internal architecture docs for:
  - codec registry and text I/O boundaries
  - generated Unicode table strategy
  - locale/global-state synchronization
  - gettext catalog parsing
  - host-limited behavior
- Add demos:
  - non-UTF-8 encode/decode
  - `open(..., encoding=...)`
  - Unicode normalization/property lookup
  - locale formatting where supported
  - gettext `.mo` translation
- Add generated Cargo dependency snapshots for all new feature combinations.
- Add panic-scan and emitted-code quality checks for codec, Unicode, locale, and gettext paths.
- Update validation lane manifests with representative fixtures.
- Close the inventory:
  - every public surface has a terminal state
  - every CPython test family has `adopted`, `adapted`, or `waived` evidence
  - every waiver has a revisit rule and regression fixture
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

- Every module surface and CPython test family in the phase inventory is closed as `done`, `intentional-diff`, `unsupported`, or `host-limited`.
- No implementation-owned source file exceeds the 900-line guardrail.
- No user-triggerable runtime panic path exists in the added stdlib/runtime surfaces.
- Global-state mutation is synchronized, waived, or host-limited with tests.

## Required Tracking Artifacts

Create and keep current during implementation:

- `issues/ad-hoc-production-text-i18n-stdlib-parity-execution.md`
- `verification/stdlib/text_i18n_parity_inventory.md`
- `verification/stdlib/text_i18n_parity_inventory.json`
- `verification/stdlib/text_i18n_parity_cpython_test_matrix.md`
- one traceability document per milestone domain under `verification/stdlib/`

The execution ledger must record:

- planning/review artifacts
- per-milestone PR links
- local validation commands and results
- CPython source/test files scanned
- adopted/adapted/waived CPython test families
- final unsupported/intentional-diff/host-limited waiver index

## Quality Contract

- Solve root causes rather than adding workaround wrappers.
- No backward-compatibility shims, legacy aliases, deprecated behavior, implicit locale-default behavior, or fallback paths may survive phase exit. Deliberate current-CPython adapters are allowed only when recorded in the inventory with Sifr-safe semantics and tests.
- No unsynchronized process-global state may be introduced.
- No data-dependent emitted `.unwrap()`, `.expect()`, or `panic!` is allowed in user runtime paths.
- Every added CPU-heavy or blocking sync function must be classified in the stdlib workload database.
- Every added external crate dependency must be represented by a stable `StdlibFeature` in `sifr_stdlib`.
- Every module added to embedded stdlib sources must have canonical `sifr.*` import-resolution tests, type-check tests, e2e pass tests, and negative diagnostics for unsupported bare CPython import forms.

## M0 Implementation Decisions To Record

1. Which encoding families are required for phase exit versus explicitly deferred?
2. Which Unicode data version is shipped, and how are generated tables produced reproducibly?
3. Which static codec aliases are shipped, and what diagnostics are used for unsupported registry mutation?
4. Which lock implementation guards locale process-global mutation, and how does it interact with threads and async tasks?
5. Which host locale names are supported in local validation?
6. What exact diagnostics and waiver evidence are recorded for unsupported `gettext.install` global mutation?
7. Which crates or generated tables meet binary-size, safety, and maintenance goals for codecs and Unicode data?

These questions must be answered in the phase execution ledger before implementing the affected milestone.

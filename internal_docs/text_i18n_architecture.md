# Text/I18n Architecture

This document records the production substrate closed by `milestone_text_i18n_5`.

## Text Invariants

Sifr `str` values are valid Unicode scalar text and lower to Rust `String`/`str`-compatible storage. Arbitrary bytes are represented as `bytes`, not text. Any API crossing between bytes and text must use the shared encoding substrate and return typed errors or recovery-carrying outcomes.

Invalid byte sequences, lone surrogates, and byte-preserving recovery are not hidden inside ordinary strings. `surrogateescape` and `surrogatepass` remain deferred behind a future explicit boundary type.

## Encoding Registry And Text I/O

`sifr.encoding`, `str.encode(...)`, `bytes.decode(...)`, `sifr.io.open_text(...)`, and builtin `open(..., encoding=..., errors=...)` share the static encoding registry. The registry is immutable; codec and error-handler registration APIs are unsupported.

The generated Cargo dependency source is `sifr_stdlib::generated_cargo_dependencies`. Text/i18n features request:

- `encoding_rs` plus `sifr_runtime` for `sifr.encoding`
- `sifr_runtime` with `unicode` for `sifr.unicode`
- `sifr_runtime` with `i18n` for `sifr.i18n`
- both runtime features when Unicode and i18n APIs are used together

Text I/O never derives an encoding from process locale state. Text-mode `open` without `encoding=` and dynamic mode strings are compile-time diagnostics.

## Unicode Data

Unicode core APIs use Unicode 17.0.0. Normalization is backed by `unicode-normalization`; names use `unicode_names2`; scalar properties, numeric data, widths, bidi values, and case folding are first-party generated tables.

Generated tables are marked generated and regenerated through `scripts/generate_unicode_tables.py`. Generated tables are excluded from hand-maintained source-size guardrails.

## Segmentation

Grapheme and word segmentation use `unicode-segmentation 1.13.3`, aligned to Unicode 17.0.0. Public APIs return owned segment text and boundary tuples. Runtime wrappers never slice user strings from untrusted byte offsets.

Sentence boundaries and streaming segmentation cursors are deferred.

## Locale And Formatting

Locale IDs, canonicalization, likely-subtag expansion/minimization, number formatting, date/time formatting, plural rules, and collation are object-scoped `sifr.i18n` APIs backed by ICU4X 2.2 compiled data.

`host_locale()` is host-limited and read-only. It may observe host environment locale values, but it does not mutate process-global state and cannot provide default text encodings.

## Translation Catalogs

`Bundle`, `Message`, and `Translator` are the stable Sifr translation API. `.mo` loading is accepted only as a backend/import format. The parser validates magic/version/table bounds, decodes declared charsets through the M1 encoding substrate, and parses plural metadata with a constrained parser rather than a general expression engine.

Empty translated forms are treated as missing so explicit fallback chains can continue.

## Panic-Free Contract

Generated code quality and runtime reviews scan encoding, Unicode, segmentation, locale, formatting, and translation paths for user-data-dependent `.unwrap(`, `.expect(`, `panic!`, `todo!`, `unimplemented!`, and `unsafe`. Fallible behavior maps to `DecodeError`, `EncodeError`, `UnicodeDataError`, `LocaleIdError`, `FormatError`, `PluralRulesError`, `CatalogError`, or `TranslationError`.

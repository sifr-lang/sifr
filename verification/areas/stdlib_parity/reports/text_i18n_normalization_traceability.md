# Text/I18n Unicode core capability Traceability

Capability: `text/i18n Unicode core`

| Pending capability | Required fixture/evidence |
| --- | --- |
| Unicode data version exposure | `sifr.unicode.data_version()` returns `17.0.0`; runtime unit test `unicode::tests::exposes_unicode_17_data_version`; e2e fixture `text_i18n_unicode_core.sifr`. |
| Normalization | `sifr.unicode.normalize` / `is_normalized` cover NFC, NFD, NFKC, and NFKD in `text_i18n_unicode_core.sifr`, backed by `unicode-normalization 0.1.25` Unicode 17.0.0 tables. |
| Properties | `text_i18n_unicode_core.sifr` covers `name`, `lookup`, `category`, `bidirectional`, `combining`, `east_asian_width`, `mirrored`, and `decomposition`, including unassigned `category == "Cn"` and empty bidi class. |
| Numeric values | `text_i18n_unicode_core.sifr` covers `decimal`, `digit`, and `numeric_value`, plus missing numeric values mapped to `UnicodeDataError`. |
| Case folding | `text_i18n_unicode_core.sifr` covers locale-insensitive full case folding for `Straße İ`; locale-sensitive mapping remains deferred to locale-formatting capability. |
| Generated table strategy | `scripts/generate_unicode_tables.py` regenerates `crates/sifr_runtime/src/unicode_data/generated.rs` from Unicode 17.0.0 `UnicodeData.txt`, `EastAsianWidth.txt`, and `CaseFolding.txt`; the generated file has an `@generated` marker, rustfmt-skipped table constants, and is excluded by `scripts/check_file_size_guardrails.py`. |
| Runtime feature gating | `sifr_runtime` exposes Unicode support only behind its `unicode` Cargo feature; `sifr.unicode` dependency emission enables that feature, while non-Unicode generated projects keep the lean runtime dependency. Targeted performance evidence: `build-project-001-additional-modules` RSS dropped below budget after the gate (`313999360` bytes versus `342556672` threshold). |

# Text/I18n translation-catalog capability Traceability

Capability: `text/i18n translation catalogs`

| Pending capability | Required fixture/evidence |
| --- | --- |
| `Bundle`, `Message`, `Translator` | `crates/sifr/tests/e2e/pass/text_i18n_translation_bundles.sifr` covers direct lookup, `Message` lookup, explicit `translator(primary).with_fallback(fallback)` chains, missing-key fallback, context lookup, and plural lookup. |
| `.mo` loader | `crates/sifr_runtime/src/i18n/translation.rs` unit tests cover little-endian catalogs, big-endian tables, declared `latin-1` charset, malformed magic, malformed string-table bounds through corrupt fixture construction, and unsupported plural syntax. |
| Safe plural parser | Runtime parser accepts the constrained gettext/C-style subset (`n`, decimal integers, `!`, arithmetic, comparisons, `&&`, `||`, ternary) and rejects unsupported tokens such as `@`; it never calls a Sifr, Python, shell, host, or general expression engine. |
| text encoding capability substrate reuse | `.mo` declared charset decoding calls `sifr_runtime::encoding::decode_text(..., "strict")`; the Latin-1 catalog fixture decodes `caf\xe9` to `café` through that path. |
| Missing catalog paths | `load_mo_file` maps `std::fs::read` failures into `CatalogError`; runtime and Sifr fixtures cover missing paths. |
| Unsupported gettext globals | Inventory entries keep `gettext.install`, global `_`, `textdomain`, and `bindtextdomain` outside the production API; no `sifr.gettext` module or global mutation surface is added. |
| Future backends | Fluent and ICU message-format backends remain deferred; `Bundle`/`Translator` expose backend-neutral lookup/fallback shape without committing to those formats. |

# Text/I18n M3 Traceability

Milestone: `milestone_text_i18n_3`

| Backlog item | Required fixture/evidence |
| --- | --- |
| `LocaleId` parsing/canonicalization | `crates/sifr/tests/e2e/pass/text_i18n_locale_formatting.sifr` covers `parse_locale("EN-US") -> "en-US"` and invalid locale typed errors; `crates/sifr_runtime/src/i18n.rs` unit tests cover the same runtime path. |
| Likely-subtag behavior | `text_i18n_locale_formatting.sifr` and runtime unit tests cover `maximize_locale("zh-CN") -> "zh-Hans-CN"` and `minimize_locale("zh-Hans-CN") -> "zh"` through ICU4X `LocaleExpander::new_extended`. |
| Read-only host locale discovery | `host_locale() -> LocaleId | None` is tested as an optional value in `text_i18n_locale_formatting.sifr`; runtime reads `LC_ALL`, `LC_MESSAGES`, and `LANG`, canonicalizes valid host values, skips `C`/`POSIX`, and never feeds text I/O defaults. |
| `NumberFormatter` | `text_i18n_locale_formatting.sifr` covers explicit `LocaleId("bn")` number formatting for `1000007`; runtime tests cover the same Bangla decimal output. |
| `DateTimeFormatter` | `text_i18n_locale_formatting.sifr` covers explicit `LocaleId("en-US")`, `DATETIME_MEDIUM`, and fixed date/time parts; runtime tests assert deterministic construction and non-empty formatted output. |
| `PluralRules` | `text_i18n_locale_formatting.sifr` covers English cardinal `one`/`other`, ordinal `two`, and invalid rule-type typed errors. |
| `Collator` | `text_i18n_locale_formatting.sifr` and runtime tests cover Spanish traditional versus English primary ordering for `pollo`/`polvo`; strength values are explicit object fields. |
| Unsupported Python locale globals | `verification/stdlib/text_i18n_substrate_inventory.md` records `locale.setlocale`, `localeconv`, `strcoll`, `strxfrm`, and implicit preferred text encodings as unsupported/deferred/rejected; `crates/sifr/tests/e2e/fail/bare_cpython_locale_import.sifr` enforces the bare namespace boundary. |

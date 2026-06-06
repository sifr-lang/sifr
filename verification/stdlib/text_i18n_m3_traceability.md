# Text/I18n M3 Traceability

Milestone: `milestone_text_i18n_3`

| Backlog item | Required fixture/evidence |
| --- | --- |
| `LocaleId` parsing/canonicalization | Deterministic locale ID fixtures with typed errors. |
| Read-only host locale discovery | Host matrix row and `Option[LocaleId]` behavior; no text I/O default use. |
| `NumberFormatter` | Explicit-locale number formatting fixtures using ICU4X compiled data. |
| `DateTimeFormatter` | Explicit-locale date/time formatting fixtures with stable inputs. |
| `PluralRules` | CLDR plural-category fixtures and typed errors. |
| `Collator` | Accepted collation behavior and host/data assumptions documented. |
| Unsupported Python locale globals | Negative fixtures or inventory entries for `setlocale`, `localeconv`, `strcoll`, `strxfrm`, and implicit preferred encoding. |

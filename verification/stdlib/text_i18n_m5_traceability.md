# Text/I18n M5 Traceability

Milestone: `milestone_text_i18n_5`

| Backlog item | Required fixture/evidence |
| --- | --- |
| Public docs | `sifr.encoding`, `sifr.unicode`, `sifr.io` explicit text I/O, and `sifr.i18n`. |
| Internal docs | Architecture updates for text invariants, registry, Unicode tables, segmentation, i18n data, catalogs, and host-limited behavior. |
| Demos | Non-UTF-8 encode/decode, explicit text open, Unicode normalization/properties, segmentation, locale formatting, translation fallback/plurals. |
| Dependency snapshots | Generated Cargo dependency snapshots for all new feature combinations. |
| Panic/emitted-code scans | Encoding, Unicode, locale, formatting, and translation generated-code quality checks. |
| Inventory closure | Every production API and CPython reference family has terminal state, stability, evidence, and revisit rule. |
| Final review | External review loop returns satisfied/pass before phase closure. |

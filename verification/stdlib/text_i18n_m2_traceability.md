# Text/I18n M2 Traceability

Milestone: `milestone_text_i18n_2`

| Backlog item | Required fixture/evidence |
| --- | --- |
| Unicode data version exposure | Runtime API and inventory entry recording exact selected table/crate versions. |
| Normalization | NFC/NFD/NFKC/NFKD fixtures adapted from CPython and Unicode data. |
| Properties | `name`, `lookup`, `category`, `bidirectional`, `combining`, `east_asian_width`, `mirrored`, `decomposition`. |
| Numeric values | `decimal`, `digit`, `numeric_value` with missing-value typed errors/options. |
| Case folding | Locale-insensitive `case_fold` fixtures; locale-sensitive mapping deferred. |
| Generated table strategy | Generated-file marker, regeneration command, panic scan, and file-size guardrail exclusion where applicable. |

# Text/I18n M2.5 Traceability

Milestone: `milestone_text_i18n_2_5`

| Backlog item | Required fixture/evidence |
| --- | --- |
| Grapheme iterator | Combining marks, emoji sequences, regional indicators, and mixed-script fixtures. |
| Grapheme indices | Boundary-safe byte index evidence; no invalid Rust string slicing. |
| Word iterator | Whitespace, punctuation, and mixed-script fixtures. |
| Word boundaries | Boundary index fixtures matching owned-string iterator behavior. |
| Deferred streaming/sentence segmentation | Inventory entries marked deferred with revisit issue. |

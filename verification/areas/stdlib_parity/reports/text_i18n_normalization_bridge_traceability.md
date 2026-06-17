# Text/I18n Unicode segmentation capability Traceability

Capability: `text/i18n Unicode segmentation`

| Pending capability | Required fixture/evidence |
| --- | --- |
| Grapheme iterator | Combining marks, emoji sequences, regional indicators, and mixed-script fixtures. |
| Grapheme indices | Boundary-safe byte index evidence; no invalid Rust string slicing. |
| Word iterator | Whitespace, punctuation, and mixed-script fixtures. |
| Word boundaries | Boundary index fixtures matching owned-string iterator behavior. |
| Deferred streaming/sentence segmentation | Inventory entries marked deferred with revisit issue. |

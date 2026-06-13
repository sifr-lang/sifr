# Text/I18n M2.5 Traceability

Milestone: `milestone_text_i18n_2_5`

| Backlog item | Required fixture/evidence |
| --- | --- |
| Grapheme segmentation | `sifr.unicode.graphemes` returns owned extended grapheme clusters; `text_i18n_unicode_segmentation.sifr` covers combining marks, emoji ZWJ sequences, and regional indicators. |
| Grapheme indices | `sifr.unicode.grapheme_indices` returns `list[tuple[int, str]]`, pairing byte start offsets with the user-perceived grapheme text; fixture covers multibyte combining-mark offsets. |
| Word extraction | `sifr.unicode.words` returns owned UAX #29 word segments; fixture covers ASCII, Greek mixed-script text, punctuation, whitespace, and numeric words. |
| Word boundaries | `sifr.unicode.word_boundaries` returns `list[tuple[int, int, str]]`, pairing byte start/end offsets with each boundary segment; fixture covers words, punctuation, and whitespace rather than exposing byte offsets alone. |
| Runtime feature gating | Segmentation is included under `sifr_runtime`'s existing `unicode` Cargo feature with `unicode-segmentation` as an optional dependency; non-Unicode generated projects keep the lean runtime dependency. |
| Unicode version alignment | Runtime unit test asserts `unicode_segmentation::UNICODE_VERSION == (17, 0, 0)`, matching the M2 Unicode 17.0.0 normalization/property/case data. |
| Deferred scope | Sentence boundaries and streaming cursor segmentation are intentionally deferred; M2.5 provides owned-string collection APIs only. |

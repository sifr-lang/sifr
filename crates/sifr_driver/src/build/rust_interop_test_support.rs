use ruff_text_size::{TextRange, TextSize};

pub(super) fn span() -> TextRange {
    TextRange::new(TextSize::from(0), TextSize::from(18))
}

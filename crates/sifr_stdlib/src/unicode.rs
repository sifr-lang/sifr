use sifr_runtime::interop::SifrIntBridge;

pub fn normalize(form: &str, text: &str) -> Result<String, String> {
    sifr_runtime::unicode::normalize(form, text)
}

pub fn is_normalized(form: &str, text: &str) -> Result<bool, String> {
    sifr_runtime::unicode::is_normalized(form, text)
}

#[must_use]
pub fn data_version() -> String {
    sifr_runtime::unicode::data_version()
}

pub fn name(text: &str) -> Result<String, String> {
    sifr_runtime::unicode::name(text)
}

pub fn lookup(name: &str) -> Result<String, String> {
    sifr_runtime::unicode::lookup(name)
}

pub fn category(text: &str) -> Result<String, String> {
    sifr_runtime::unicode::category(text)
}

pub fn bidirectional(text: &str) -> Result<String, String> {
    sifr_runtime::unicode::bidirectional(text)
}

pub fn combining(text: &str) -> Result<SifrIntBridge, String> {
    sifr_runtime::unicode::combining(text).map(SifrIntBridge::from)
}

pub fn east_asian_width(text: &str) -> Result<String, String> {
    sifr_runtime::unicode::east_asian_width(text)
}

pub fn mirrored(text: &str) -> Result<bool, String> {
    sifr_runtime::unicode::mirrored(text)
}

pub fn decomposition(text: &str) -> Result<String, String> {
    sifr_runtime::unicode::decomposition(text)
}

pub fn decimal(text: &str) -> Result<SifrIntBridge, String> {
    sifr_runtime::unicode::decimal(text).map(SifrIntBridge::from)
}

pub fn digit(text: &str) -> Result<SifrIntBridge, String> {
    sifr_runtime::unicode::digit(text).map(SifrIntBridge::from)
}

pub fn numeric_value(text: &str) -> Result<f64, String> {
    sifr_runtime::unicode::numeric_value(text)
}

#[must_use]
pub fn case_fold(text: &str) -> String {
    sifr_runtime::unicode::case_fold(text)
}

#[must_use]
pub fn graphemes(text: &str) -> Vec<String> {
    sifr_runtime::unicode::graphemes(text)
}

#[must_use]
fn grapheme_indices(text: &str) -> Vec<(i64, String)> {
    sifr_runtime::unicode::grapheme_indices(text)
}

#[must_use]
pub fn grapheme_indices_flat(text: &str) -> Vec<String> {
    grapheme_indices(text)
        .into_iter()
        .flat_map(|(index, value)| [index.to_string(), value])
        .collect()
}

#[must_use]
pub fn words(text: &str) -> Vec<String> {
    sifr_runtime::unicode::words(text)
}

#[must_use]
fn word_boundaries(text: &str) -> Vec<(i64, i64, String)> {
    sifr_runtime::unicode::word_boundaries(text)
}

#[must_use]
pub fn word_boundaries_flat(text: &str) -> Vec<String> {
    word_boundaries(text)
        .into_iter()
        .flat_map(|(start, end, value)| [start.to_string(), end.to_string(), value])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        bidirectional, case_fold, category, combining, data_version, decimal, decomposition, digit,
        east_asian_width, grapheme_indices, grapheme_indices_flat, graphemes, is_normalized,
        lookup, mirrored, name, normalize, numeric_value, word_boundaries, word_boundaries_flat,
        words,
    };

    #[test]
    fn unicode_adapter_delegates_full_text_surface() {
        let normalized = normalize("NFC", "e\u{301}").expect("NFC normalization");
        assert_eq!(normalized, "\u{E9}");
        assert!(is_normalized("NFC", &normalized).expect("normalization check"));
        assert_eq!(data_version(), "17.0.0");

        assert_eq!(name("\u{2603}").expect("snowman name"), "SNOWMAN");
        assert_eq!(lookup("snowman").expect("snowman lookup"), "\u{2603}");
        assert_eq!(category("A").expect("category"), "Lu");
        assert_eq!(bidirectional("A").expect("bidi"), "L");
        assert_eq!(
            combining("\u{301}").expect("combining").to_i64_saturating(),
            230
        );
        assert_eq!(east_asian_width("\u{3042}").expect("width"), "W");
        assert!(!mirrored("A").expect("mirrored"));
        assert_eq!(decomposition("\u{212B}").expect("decomp"), "00C5");
        assert_eq!(digit("7").expect("digit").to_i64_saturating(), 7);
        assert_eq!(numeric_value("\u{00BD}").expect("numeric"), 0.5);
        assert!(decimal("A").is_err());

        assert_eq!(case_fold("Stra\u{DF}e \u{130}"), "strasse i\u{307}");
        assert_eq!(
            graphemes("a\u{301}\u{1F469}\u{200D}\u{1F680}"),
            vec!["a\u{301}", "\u{1F469}\u{200D}\u{1F680}"]
        );
        assert_eq!(
            grapheme_indices("a\u{301}b"),
            vec![(0, "a\u{301}".to_string()), (3, "b".to_string())]
        );
        assert_eq!(
            grapheme_indices_flat("a\u{301}b"),
            vec!["0", "a\u{301}", "3", "b"]
        );
        assert_eq!(words("Hi, κόσμε!").as_slice(), ["Hi", "κόσμε"]);
        assert_eq!(word_boundaries("Hi, κόσμε!")[0], (0, 2, "Hi".to_string()));
        assert_eq!(&word_boundaries_flat("Hi, κόσμε!")[..3], ["0", "2", "Hi"]);
    }
}

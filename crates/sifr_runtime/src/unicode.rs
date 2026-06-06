//! Unicode normalization, data, and case mapping helpers for generated code.

use unicode_normalization::{is_nfc, is_nfd, is_nfkc, is_nfkd, UnicodeNormalization};

use crate::unicode_data;

const DEFAULT_CATEGORY: &str = "Cn";
const DEFAULT_BIDI_CLASS: &str = "";
const DEFAULT_EAST_ASIAN_WIDTH: &str = "N";

#[must_use]
pub fn data_version() -> String {
    unicode_data::UNICODE_DATA_VERSION.to_string()
}

pub fn normalize(form: &str, text: &str) -> Result<String, String> {
    match canonical_form(form)? {
        "NFC" => Ok(text.nfc().collect()),
        "NFD" => Ok(text.nfd().collect()),
        "NFKC" => Ok(text.nfkc().collect()),
        "NFKD" => Ok(text.nfkd().collect()),
        _ => Err(format!("unsupported normalization form '{form}'")),
    }
}

pub fn is_normalized(form: &str, text: &str) -> Result<bool, String> {
    match canonical_form(form)? {
        "NFC" => Ok(is_nfc(text)),
        "NFD" => Ok(is_nfd(text)),
        "NFKC" => Ok(is_nfkc(text)),
        "NFKD" => Ok(is_nfkd(text)),
        _ => Err(format!("unsupported normalization form '{form}'")),
    }
}

pub fn name(text: &str) -> Result<String, String> {
    let ch = single_scalar(text)?;
    unicode_names2::name(ch)
        .map(|name| name.to_string())
        .ok_or_else(|| format!("no Unicode name for U+{:04X}", u32::from(ch)))
}

pub fn lookup(name: &str) -> Result<String, String> {
    unicode_names2::character(name)
        .map(|ch| ch.to_string())
        .ok_or_else(|| format!("unknown Unicode character name '{name}'"))
}

pub fn category(text: &str) -> Result<String, String> {
    let ch = single_scalar(text)?;
    Ok(property_record(ch)
        .map_or(DEFAULT_CATEGORY, |record| record.category)
        .to_string())
}

pub fn bidirectional(text: &str) -> Result<String, String> {
    let ch = single_scalar(text)?;
    Ok(property_record(ch)
        .map_or(DEFAULT_BIDI_CLASS, |record| record.bidi)
        .to_string())
}

pub fn combining(text: &str) -> Result<i64, String> {
    let ch = single_scalar(text)?;
    Ok(property_record(ch).map_or(0, |record| i64::from(record.combining)))
}

pub fn east_asian_width(text: &str) -> Result<String, String> {
    let ch = single_scalar(text)?;
    Ok(width_record(ch)
        .unwrap_or(DEFAULT_EAST_ASIAN_WIDTH)
        .to_string())
}

pub fn mirrored(text: &str) -> Result<bool, String> {
    let ch = single_scalar(text)?;
    Ok(property_record(ch).is_some_and(|record| record.mirrored))
}

pub fn decomposition(text: &str) -> Result<String, String> {
    let ch = single_scalar(text)?;
    Ok(property_record(ch)
        .map_or("", |record| record.decomposition)
        .to_string())
}

pub fn decimal(text: &str) -> Result<i64, String> {
    let ch = single_scalar(text)?;
    let Some(record) = property_record(ch) else {
        return Err(format!("no decimal value for U+{:04X}", u32::from(ch)));
    };
    if record.decimal < 0 {
        return Err(format!("no decimal value for U+{:04X}", u32::from(ch)));
    }
    Ok(i64::from(record.decimal))
}

pub fn digit(text: &str) -> Result<i64, String> {
    let ch = single_scalar(text)?;
    let Some(record) = property_record(ch) else {
        return Err(format!("no digit value for U+{:04X}", u32::from(ch)));
    };
    if record.digit < 0 {
        return Err(format!("no digit value for U+{:04X}", u32::from(ch)));
    }
    Ok(i64::from(record.digit))
}

pub fn numeric_value(text: &str) -> Result<f64, String> {
    let ch = single_scalar(text)?;
    let Some(record) = property_record(ch) else {
        return Err(format!("no numeric value for U+{:04X}", u32::from(ch)));
    };
    parse_numeric(record.numeric)
        .ok_or_else(|| format!("no numeric value for U+{:04X}", u32::from(ch)))
}

#[must_use]
pub fn case_fold(text: &str) -> String {
    let mut folded = String::new();
    for ch in text.chars() {
        if let Some(mapping) = case_fold_mapping(ch) {
            folded.push_str(mapping);
        } else {
            folded.push(ch);
        }
    }
    folded
}

fn canonical_form(form: &str) -> Result<&'static str, String> {
    match form {
        "NFC" | "nfc" => Ok("NFC"),
        "NFD" | "nfd" => Ok("NFD"),
        "NFKC" | "nfkc" => Ok("NFKC"),
        "NFKD" | "nfkd" => Ok("NFKD"),
        _ => Err(format!("unsupported normalization form '{form}'")),
    }
}

fn single_scalar(text: &str) -> Result<char, String> {
    let mut chars = text.chars();
    let Some(ch) = chars.next() else {
        return Err("expected exactly one Unicode scalar, got empty string".to_string());
    };
    if chars.next().is_some() {
        return Err("expected exactly one Unicode scalar, got multiple scalars".to_string());
    }
    Ok(ch)
}

fn property_record(ch: char) -> Option<&'static unicode_data::UnicodePropertyRecord> {
    let codepoint = u32::from(ch);
    let idx = unicode_data::PROPERTY_RANGES.partition_point(|record| record.end < codepoint);
    unicode_data::PROPERTY_RANGES
        .get(idx)
        .filter(|record| record.start <= codepoint && codepoint <= record.end)
}

fn width_record(ch: char) -> Option<&'static str> {
    let codepoint = u32::from(ch);
    let idx =
        unicode_data::EAST_ASIAN_WIDTH_RANGES.partition_point(|record| record.end < codepoint);
    unicode_data::EAST_ASIAN_WIDTH_RANGES
        .get(idx)
        .filter(|record| record.start <= codepoint && codepoint <= record.end)
        .map(|record| record.width)
}

fn case_fold_mapping(ch: char) -> Option<&'static str> {
    let codepoint = u32::from(ch);
    let idx = unicode_data::CASE_FOLDING
        .binary_search_by_key(&codepoint, |mapping| mapping.codepoint)
        .ok()?;
    Some(unicode_data::CASE_FOLDING[idx].mapping)
}

fn parse_numeric(value: &str) -> Option<f64> {
    if value.is_empty() {
        return None;
    }
    if let Some((numerator, denominator)) = value.split_once('/') {
        let numerator = numerator.parse::<f64>().ok()?;
        let denominator = denominator.parse::<f64>().ok()?;
        if denominator == 0.0 {
            return None;
        }
        return Some(numerator / denominator);
    }
    value.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::{
        case_fold, category, data_version, decomposition, east_asian_width, lookup, name,
        normalize, numeric_value,
    };

    #[test]
    fn exposes_unicode_17_data_version() {
        assert_eq!(data_version(), "17.0.0");
    }

    #[test]
    fn normalizes_and_queries_properties() {
        assert_eq!(normalize("NFC", "e\u{301}").unwrap(), "\u{E9}");
        assert_eq!(name("\u{2603}").unwrap(), "SNOWMAN");
        assert_eq!(lookup("snowman").unwrap(), "\u{2603}");
        assert_eq!(category("A").unwrap(), "Lu");
        assert_eq!(east_asian_width("\u{3042}").unwrap(), "W");
        assert_eq!(decomposition("\u{212B}").unwrap(), "00C5");
        assert_eq!(numeric_value("\u{00BD}").unwrap(), 0.5);
        assert_eq!(case_fold("Stra\u{DF}e \u{130}"), "strasse i\u{307}");
    }
}

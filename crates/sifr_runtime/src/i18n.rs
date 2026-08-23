//! ICU4X-backed locale identifiers, formatting, plural rules, collation, and
//! translation catalog compatibility backends.

use std::cmp::Ordering;
use std::str::FromStr;

mod translation;

use icu_collator::{
    Collator,
    options::{CollatorOptions, Strength},
};
use icu_datetime::{
    DateTimeFormatter, fieldsets,
    input::{Date, DateTime, Time},
};
use icu_decimal::{DecimalFormatter, input::Decimal};
use icu_locale::{Locale, LocaleCanonicalizer, LocaleExpander};
use icu_plurals::{
    PluralCategory, PluralOperands, PluralRuleType, PluralRules, PluralRulesOptions,
};

pub fn canonicalize_locale(locale: &str) -> Result<String, String> {
    let mut locale = parse_locale(locale)?;
    let canonicalizer = LocaleCanonicalizer::new_extended();
    canonicalizer.canonicalize(&mut locale);
    Ok(locale.to_string())
}

pub fn maximize_locale(locale: &str) -> Result<String, String> {
    let mut locale = parse_locale(locale)?;
    let expander = LocaleExpander::new_extended();
    expander.maximize(&mut locale.id);
    Ok(locale.to_string())
}

pub fn minimize_locale(locale: &str) -> Result<String, String> {
    let mut locale = parse_locale(locale)?;
    let expander = LocaleExpander::new_extended();
    expander.minimize(&mut locale.id);
    Ok(locale.to_string())
}

#[must_use]
pub fn host_locale() -> Option<String> {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .into_iter()
        .filter_map(std::env::var_os)
        .filter_map(|value| value.into_string().ok())
        .filter_map(|value| canonicalize_host_locale_token(&value))
        .find_map(|locale| canonicalize_locale(&locale).ok())
}

pub fn format_number(locale: &str, value: &str) -> Result<String, String> {
    let locale = parse_locale(locale)?;
    let locale_label = locale.to_string();
    let decimal = Decimal::from_str(value)
        .map_err(|err| format!("invalid decimal value '{value}': {err}"))?;
    let formatter = DecimalFormatter::try_new(locale.into(), Default::default())
        .map_err(|err| format!("unsupported number locale '{locale_label}': {err}"))?;
    Ok(formatter.format(&decimal).to_string())
}

#[allow(clippy::too_many_arguments)]
pub fn format_datetime(
    locale: &str,
    style: &str,
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
) -> Result<String, String> {
    let locale = parse_locale(locale)?;
    let locale_label = locale.to_string();
    let date = Date::try_new_iso(
        i32::try_from(year).map_err(|_| format!("year out of range: {year}"))?,
        u8::try_from(month).map_err(|_| format!("month out of range: {month}"))?,
        u8::try_from(day).map_err(|_| format!("day out of range: {day}"))?,
    )
    .map_err(|err| format!("invalid ISO date: {err}"))?;
    let time = Time::try_new(
        u8::try_from(hour).map_err(|_| format!("hour out of range: {hour}"))?,
        u8::try_from(minute).map_err(|_| format!("minute out of range: {minute}"))?,
        u8::try_from(second).map_err(|_| format!("second out of range: {second}"))?,
        0,
    )
    .map_err(|err| format!("invalid time: {err}"))?;
    let value = DateTime { date, time };
    match style {
        "short" => {
            let formatter = DateTimeFormatter::try_new(locale.into(), fieldsets::YMDT::short())
                .map_err(|err| format!("unsupported date/time locale '{locale_label}': {err}"))?;
            Ok(formatter.format(&value).to_string())
        }
        "medium" => {
            let formatter = DateTimeFormatter::try_new(locale.into(), fieldsets::YMDT::medium())
                .map_err(|err| format!("unsupported date/time locale '{locale_label}': {err}"))?;
            Ok(formatter.format(&value).to_string())
        }
        "long" => {
            let formatter = DateTimeFormatter::try_new(locale.into(), fieldsets::YMDT::long())
                .map_err(|err| format!("unsupported date/time locale '{locale_label}': {err}"))?;
            Ok(formatter.format(&value).to_string())
        }
        _ => Err(format!("unsupported date/time style '{style}'")),
    }
}

pub fn plural_category(locale: &str, rule_type: &str, value: &str) -> Result<String, String> {
    let locale = parse_locale(locale)?;
    let locale_label = locale.to_string();
    let decimal = Decimal::from_str(value)
        .map_err(|err| format!("invalid plural operand '{value}': {err}"))?;
    let operands = PluralOperands::from(&decimal);
    let plural_type = match rule_type {
        "cardinal" => PluralRuleType::Cardinal,
        "ordinal" => PluralRuleType::Ordinal,
        _ => return Err(format!("unsupported plural rule type '{rule_type}'")),
    };
    let options = PluralRulesOptions::default().with_type(plural_type);
    let rules = PluralRules::try_new(locale.into(), options)
        .map_err(|err| format!("unsupported plural locale '{locale_label}': {err}"))?;
    Ok(category_label(rules.category_for(operands)).to_string())
}

pub fn collate(locale: &str, strength: &str, left: &str, right: &str) -> Result<i64, String> {
    let locale = parse_locale(locale)?;
    let locale_label = locale.to_string();
    let mut options = CollatorOptions::default();
    options.strength = match strength {
        "default" => None,
        "primary" => Some(Strength::Primary),
        "secondary" => Some(Strength::Secondary),
        "tertiary" => Some(Strength::Tertiary),
        _ => return Err(format!("unsupported collation strength '{strength}'")),
    };
    let collator = Collator::try_new(locale.into(), options)
        .map_err(|err| format!("unsupported collation locale '{locale_label}': {err}"))?;
    Ok(match collator.compare(left, right) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    })
}

pub fn validate_mo_catalog(data: &[u8]) -> Result<String, String> {
    translation::Catalog::parse(data).map(|_| "ok".to_string())
}

pub fn read_mo_catalog_file(path: &str) -> Result<Vec<u8>, String> {
    let data = std::fs::read(path).map_err(|err| format!("failed to read .mo catalog: {err}"))?;
    translation::Catalog::parse(&data)?;
    Ok(data)
}

pub fn mo_lookup(data: &[u8], message_id: &str) -> Result<Option<String>, String> {
    let catalog = translation::Catalog::parse(data)?;
    Ok(catalog.lookup(None, message_id))
}

pub fn mo_lookup_context(
    data: &[u8],
    context: &str,
    message_id: &str,
) -> Result<Option<String>, String> {
    let catalog = translation::Catalog::parse(data)?;
    Ok(catalog.lookup(Some(context), message_id))
}

pub fn mo_lookup_plural(
    data: &[u8],
    singular: &str,
    plural: &str,
    count: i64,
) -> Result<Option<String>, String> {
    let catalog = translation::Catalog::parse(data)?;
    catalog.lookup_plural(None, singular, plural, count)
}

pub fn mo_lookup_context_plural(
    data: &[u8],
    context: &str,
    singular: &str,
    plural: &str,
    count: i64,
) -> Result<Option<String>, String> {
    let catalog = translation::Catalog::parse(data)?;
    catalog.lookup_plural(Some(context), singular, plural, count)
}

fn parse_locale(locale: &str) -> Result<Locale, String> {
    Locale::try_from_str(locale)
        .map_err(|err| format!("invalid locale identifier '{locale}': {err}"))
}

fn canonicalize_host_locale_token(value: &str) -> Option<String> {
    let token = value
        .split(':')
        .find(|candidate| !candidate.trim().is_empty())?
        .trim();
    if token.eq_ignore_ascii_case("C") || token.eq_ignore_ascii_case("POSIX") {
        return None;
    }
    let without_modifier = token.split('@').next().unwrap_or(token);
    let without_codeset = without_modifier
        .split('.')
        .next()
        .unwrap_or(without_modifier);
    let locale = without_codeset.replace('_', "-");
    if locale.is_empty() {
        None
    } else {
        Some(locale)
    }
}

fn category_label(category: PluralCategory) -> &'static str {
    match category {
        PluralCategory::Zero => "zero",
        PluralCategory::One => "one",
        PluralCategory::Two => "two",
        PluralCategory::Few => "few",
        PluralCategory::Many => "many",
        PluralCategory::Other => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        canonicalize_locale, collate, format_datetime, format_number, maximize_locale,
        minimize_locale, mo_lookup, mo_lookup_context, mo_lookup_context_plural, mo_lookup_plural,
        plural_category, read_mo_catalog_file, validate_mo_catalog,
    };
    use std::io::Write as _;

    #[test]
    fn locale_ids_are_canonicalized_and_expanded() {
        assert_eq!(canonicalize_locale("EN-US").as_deref(), Ok("en-US"));
        assert_eq!(maximize_locale("zh-CN").as_deref(), Ok("zh-Hans-CN"));
        assert_eq!(minimize_locale("zh-Hans-CN").as_deref(), Ok("zh"));
        assert!(canonicalize_locale("not a locale").is_err());
    }

    #[test]
    fn number_formatting_uses_explicit_locale_data() {
        assert_eq!(
            format_number("bn", "1000007").as_deref(),
            Ok("\u{9E7}\u{9E6},\u{9E6}\u{9E6},\u{9E6}\u{9E6}\u{9ED}")
        );
    }

    #[test]
    fn date_time_formatting_uses_formatter_objects() {
        let formatted = format_datetime("en-US", "medium", 2025, 1, 15, 16, 9, 35)
            .expect("date/time formatting should succeed");

        assert!(formatted.contains("2025"));
        assert!(formatted.contains("Jan"));
    }

    #[test]
    fn plural_rules_return_stable_category_labels() {
        assert_eq!(plural_category("en", "cardinal", "1").as_deref(), Ok("one"));
        assert_eq!(
            plural_category("en", "cardinal", "2").as_deref(),
            Ok("other")
        );
        assert_eq!(plural_category("en", "ordinal", "2").as_deref(), Ok("two"));
    }

    #[test]
    fn collation_uses_explicit_locale_and_strength() {
        assert_eq!(collate("en", "primary", "resume", "resume"), Ok(0));
        assert!(
            collate("es-u-co-trad", "primary", "pollo", "polvo")
                .expect("Spanish traditional collation should compare")
                > 0
        );
        assert!(
            collate("en", "primary", "pollo", "polvo")
                .expect("English primary collation should compare")
                < 0
        );
    }

    #[test]
    fn mo_backend_is_exposed_through_runtime_api() {
        let data = test_mo_bytes(
            &[
                (
                    b"",
                    b"Content-Type: text/plain; charset=utf-8\nPlural-Forms: nplurals=2; plural=n != 1;\n",
                ),
                (b"hello", b"bonjour"),
                (b"menu\x04open", b"ouvrir"),
                (b"file\x00files", b"fichier\x00fichiers"),
            ],
            false,
        );

        assert_eq!(validate_mo_catalog(&data).as_deref(), Ok("ok"));
        assert_eq!(
            mo_lookup(&data, "hello").expect("lookup").as_deref(),
            Some("bonjour")
        );
        assert_eq!(
            mo_lookup_context(&data, "menu", "open")
                .expect("context lookup")
                .as_deref(),
            Some("ouvrir")
        );
        assert_eq!(
            mo_lookup_plural(&data, "file", "files", 2)
                .expect("plural lookup")
                .as_deref(),
            Some("fichiers")
        );
        assert_eq!(
            mo_lookup_context_plural(&data, "missing", "file", "files", 2)
                .expect("missing context plural"),
            None
        );
    }

    #[test]
    fn mo_file_loader_reports_missing_paths() {
        let missing = read_mo_catalog_file("/tmp/sifr_i18n_missing_catalog_for_test.mo")
            .expect_err("missing catalog path should fail");
        assert!(missing.contains("failed to read .mo catalog"));
    }

    #[test]
    fn mo_file_loader_reads_and_validates_catalogs() {
        let data = test_mo_bytes(&[(b"hello", b"bonjour")], false);
        let path = std::env::temp_dir().join("sifr_i18n_runtime_catalog_test.mo");
        let mut file = std::fs::File::create(&path).expect("create temp catalog");
        file.write_all(&data).expect("write temp catalog");

        let loaded = read_mo_catalog_file(path.to_str().expect("utf-8 temp path"))
            .expect("catalog file should load");

        assert_eq!(
            mo_lookup(&loaded, "hello").expect("lookup").as_deref(),
            Some("bonjour")
        );
        std::fs::remove_file(path).expect("remove temp catalog");
    }

    fn test_mo_bytes(entries: &[(&[u8], &[u8])], big_endian: bool) -> Vec<u8> {
        let count = entries.len();
        let original_table = 28usize;
        let translated_table = original_table + count * 8;
        let mut cursor = translated_table + count * 8;
        let mut original_records = Vec::new();
        let mut translated_records = Vec::new();
        let mut payload = Vec::new();
        for (original, translated) in entries {
            original_records.push((*original, cursor));
            payload.extend_from_slice(original);
            payload.push(0);
            cursor += original.len() + 1;
            translated_records.push((*translated, cursor));
            payload.extend_from_slice(translated);
            payload.push(0);
            cursor += translated.len() + 1;
        }

        let mut data = Vec::new();
        if big_endian {
            data.extend_from_slice(&[0x95, 0x04, 0x12, 0xDE]);
        } else {
            data.extend_from_slice(&[0xDE, 0x12, 0x04, 0x95]);
        }
        push_u32(&mut data, 0, big_endian);
        push_u32(
            &mut data,
            u32::try_from(count).expect("test catalog entry count should fit u32"),
            big_endian,
        );
        push_u32(
            &mut data,
            u32::try_from(original_table).expect("test original table offset should fit u32"),
            big_endian,
        );
        push_u32(
            &mut data,
            u32::try_from(translated_table).expect("test translated table offset should fit u32"),
            big_endian,
        );
        push_u32(&mut data, 0, big_endian);
        push_u32(&mut data, 0, big_endian);
        for (text, offset) in original_records {
            push_u32(
                &mut data,
                u32::try_from(text.len()).expect("test original length should fit u32"),
                big_endian,
            );
            push_u32(
                &mut data,
                u32::try_from(offset).expect("test original offset should fit u32"),
                big_endian,
            );
        }
        for (text, offset) in translated_records {
            push_u32(
                &mut data,
                u32::try_from(text.len()).expect("test translation length should fit u32"),
                big_endian,
            );
            push_u32(
                &mut data,
                u32::try_from(offset).expect("test translation offset should fit u32"),
                big_endian,
            );
        }
        data.extend_from_slice(&payload);
        data
    }

    fn push_u32(data: &mut Vec<u8>, value: u32, big_endian: bool) {
        if big_endian {
            data.extend_from_slice(&value.to_be_bytes());
        } else {
            data.extend_from_slice(&value.to_le_bytes());
        }
    }
}

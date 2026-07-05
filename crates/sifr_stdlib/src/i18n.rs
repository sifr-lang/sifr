use sifr_runtime::interop::SifrIntBridge;

#[must_use]
pub const fn feature_name() -> &'static str {
    "i18n"
}

pub fn i18n_locale_canonicalize(locale: &str) -> Result<String, String> {
    sifr_runtime::i18n::canonicalize_locale(locale)
}

pub fn i18n_locale_maximize(locale: &str) -> Result<String, String> {
    sifr_runtime::i18n::maximize_locale(locale)
}

pub fn i18n_locale_minimize(locale: &str) -> Result<String, String> {
    sifr_runtime::i18n::minimize_locale(locale)
}

#[must_use]
pub fn i18n_host_locale() -> Option<String> {
    sifr_runtime::i18n::host_locale()
}

pub fn i18n_format_number(locale: &str, value: &str) -> Result<String, String> {
    sifr_runtime::i18n::format_number(locale, value)
}

#[allow(clippy::too_many_arguments)]
pub fn i18n_format_datetime(
    locale: &str,
    style: &str,
    year: SifrIntBridge,
    month: SifrIntBridge,
    day: SifrIntBridge,
    hour: SifrIntBridge,
    minute: SifrIntBridge,
    second: SifrIntBridge,
) -> Result<String, String> {
    sifr_runtime::i18n::format_datetime(
        locale,
        style,
        year.to_i64_saturating(),
        month.to_i64_saturating(),
        day.to_i64_saturating(),
        hour.to_i64_saturating(),
        minute.to_i64_saturating(),
        second.to_i64_saturating(),
    )
}

pub fn i18n_plural_category(locale: &str, rule_type: &str, value: &str) -> Result<String, String> {
    sifr_runtime::i18n::plural_category(locale, rule_type, value)
}

pub fn i18n_collate(
    locale: &str,
    strength: &str,
    left: &str,
    right: &str,
) -> Result<SifrIntBridge, String> {
    sifr_runtime::i18n::collate(locale, strength, left, right).map(SifrIntBridge::from)
}

pub fn i18n_mo_validate(catalog: &[u8]) -> Result<String, String> {
    sifr_runtime::i18n::validate_mo_catalog(catalog)
}

pub fn i18n_mo_load_file(path: &str) -> Result<Vec<u8>, String> {
    sifr_runtime::i18n::read_mo_catalog_file(path)
}

pub fn i18n_mo_lookup(catalog: &[u8], message_id: &str) -> Result<Option<String>, String> {
    sifr_runtime::i18n::mo_lookup(catalog, message_id)
}

pub fn i18n_mo_lookup_context(
    catalog: &[u8],
    context: &str,
    message_id: &str,
) -> Result<Option<String>, String> {
    sifr_runtime::i18n::mo_lookup_context(catalog, context, message_id)
}

pub fn i18n_mo_lookup_plural(
    catalog: &[u8],
    singular: &str,
    plural: &str,
    count: SifrIntBridge,
) -> Result<Option<String>, String> {
    sifr_runtime::i18n::mo_lookup_plural(catalog, singular, plural, count.to_i64_saturating())
}

pub fn i18n_mo_lookup_context_plural(
    catalog: &[u8],
    context: &str,
    singular: &str,
    plural: &str,
    count: SifrIntBridge,
) -> Result<Option<String>, String> {
    sifr_runtime::i18n::mo_lookup_context_plural(
        catalog,
        context,
        singular,
        plural,
        count.to_i64_saturating(),
    )
}

pub fn canonicalize_locale(locale: &str) -> Result<String, String> {
    i18n_locale_canonicalize(locale)
}

pub fn format_number(locale: &str, value: &str) -> Result<String, String> {
    i18n_format_number(locale, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i18n_adapter_delegates_locale_and_formatting_surface() {
        assert_eq!(feature_name(), "i18n");
        assert_eq!(i18n_locale_canonicalize("EN-us").unwrap(), "en-US");
        assert_eq!(i18n_locale_maximize("zh-CN").unwrap(), "zh-Hans-CN");
        assert_eq!(i18n_locale_minimize("zh-Hans-CN").unwrap(), "zh");

        let formatted = i18n_format_number("en-US", "12345.5").unwrap();
        assert!(formatted.contains("12"));

        let date = i18n_format_datetime(
            "en-US",
            "medium",
            SifrIntBridge::from(2025),
            SifrIntBridge::from(1),
            SifrIntBridge::from(15),
            SifrIntBridge::from(16),
            SifrIntBridge::from(9),
            SifrIntBridge::from(35),
        )
        .unwrap();
        assert!(date.contains("2025"));

        assert_eq!(i18n_plural_category("en", "cardinal", "1").unwrap(), "one");
        assert_eq!(
            i18n_collate("en", "primary", "resume", "resume")
                .unwrap()
                .to_i64_saturating(),
            0
        );
    }
}

pub fn canonicalize_locale(locale: &str) -> Result<String, String> {
    sifr_runtime::i18n::canonicalize_locale(locale)
}

pub fn format_number(locale: &str, value: &str) -> Result<String, String> {
    sifr_runtime::i18n::format_number(locale, value)
}

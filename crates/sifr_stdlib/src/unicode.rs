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

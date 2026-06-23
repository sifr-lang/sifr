#[test]
fn crate_identity_is_generated_program_stdlib() {
    assert_eq!(sifr_stdlib::crate_identity(), "sifr_stdlib");
    assert!(sifr_stdlib::feature_contract::leaf_features().contains(&"json"));
    assert!(sifr_stdlib::feature_contract::leaf_features().contains(&"http"));
}

#[cfg(feature = "json")]
#[test]
fn json_wrapper_shapes_digit_limit_errors() {
    assert_eq!(sifr_stdlib::json::default_integer_digit_limit(), 4_096);
    let error = sifr_stdlib::json::validate_integer_digit_limit("{\"n\": 1234}", 3)
        .expect_err("digit limit should reject a four-digit integer");
    assert!(error.contains("exceeding limit 3"));
}

#[cfg(feature = "unicode")]
#[test]
fn unicode_wrapper_normalizes_text() {
    let normalized = sifr_stdlib::unicode::normalize("NFC", "e\u{301}").expect("NFC normalization");
    assert_eq!(normalized, "\u{e9}");
    assert!(sifr_stdlib::unicode::is_normalized("NFC", &normalized).expect("normalization check"));
    assert!(!sifr_stdlib::unicode::data_version().is_empty());
}

#[cfg(feature = "i18n")]
#[test]
fn i18n_wrapper_canonicalizes_locale() {
    let locale = sifr_stdlib::i18n::canonicalize_locale("EN-us").expect("canonical locale");
    assert_eq!(locale, "en-US");
    let formatted = sifr_stdlib::i18n::format_number("en-US", "12345.5").expect("formatted number");
    assert!(formatted.contains("12"));
}

#[cfg(feature = "http")]
#[test]
fn http_header_name_canonicalizes_and_rejects_invalid_names() {
    let header = sifr_stdlib::http::HeaderName::new("Content-Type").expect("valid header");
    assert_eq!(header.as_str(), "content-type");
    assert!(sifr_stdlib::http::HeaderName::new("bad header").is_err());
    assert!(sifr_stdlib::http::HeaderName::new("").is_err());
    assert!(sifr_stdlib::http::HeaderName::new("x-\u{e9}").is_err());
}

#[cfg(feature = "html")]
#[test]
fn html_leaf_escapes_and_unescapes_common_entities() {
    assert_eq!(
        sifr_stdlib::html::html_escape("<a href=\"x\">'ok' & done</a>"),
        "&lt;a href=&quot;x&quot;&gt;&#x27;ok&#x27; &amp; done&lt;/a&gt;"
    );
    assert_eq!(
        sifr_stdlib::html::html_unescape("&lt;b&gt;safe &amp; sound&lt;/b&gt;"),
        "<b>safe & sound</b>"
    );
    assert_eq!(sifr_stdlib::html::feature_name(), "html");
}

#[cfg(feature = "platform")]
#[test]
fn platform_leaf_returns_non_empty_host_strings() {
    assert!(!sifr_stdlib::platform::platform_system().is_empty());
    assert!(!sifr_stdlib::platform::platform_arch().is_empty());
    assert!(!sifr_stdlib::platform::platform_node().is_empty());
    assert!(!sifr_stdlib::platform::platform_processor().is_empty());
    assert_eq!(sifr_stdlib::platform::feature_name(), "platform");
}

#[cfg(feature = "runtime-observability")]
#[test]
fn runtime_observability_emits_diagnostic_without_subscriber() {
    sifr_stdlib::runtime_observability::emit_diagnostic(
        tracing::Level::INFO,
        "test-target",
        "test-diagnostic",
        "test message",
    );
}

#[cfg(all(
    feature = "base64",
    feature = "fs",
    feature = "gzip",
    feature = "hash",
    feature = "html",
    feature = "net",
    feature = "platform",
    feature = "process",
    feature = "python",
    feature = "regex",
    feature = "signals",
    feature = "tls",
    feature = "toml",
    feature = "url",
    feature = "uuid",
    feature = "zipfile"
))]
#[test]
fn marker_modules_report_leaf_names() {
    let markers = [
        sifr_stdlib::base64::feature_name(),
        sifr_stdlib::fs::feature_name(),
        sifr_stdlib::gzip::feature_name(),
        sifr_stdlib::hash::feature_name(),
        sifr_stdlib::html::feature_name(),
        sifr_stdlib::net::feature_name(),
        sifr_stdlib::platform::feature_name(),
        sifr_stdlib::process::feature_name(),
        sifr_stdlib::python::feature_name(),
        sifr_stdlib::regex::feature_name(),
        sifr_stdlib::signals::feature_name(),
        sifr_stdlib::tls::feature_name(),
        sifr_stdlib::toml::feature_name(),
        sifr_stdlib::url::feature_name(),
        sifr_stdlib::uuid::feature_name(),
        sifr_stdlib::zipfile::feature_name(),
    ];
    for marker in markers {
        assert!(sifr_stdlib::feature_contract::leaf_features().contains(&marker));
    }
}

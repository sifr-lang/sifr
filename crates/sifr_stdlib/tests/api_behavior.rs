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

#[cfg(feature = "url")]
#[test]
fn url_leaf_matches_public_url_helpers() {
    use sifr_runtime::interop::SifrIntBridge;

    let parsed =
        sifr_stdlib::url::url_parse_parts("https://user:pass@example.com:8443/a/../b?x=1&x=2#frag")
            .unwrap_or_else(|err| panic!("valid URL: {err}"));
    assert_eq!(parsed[0], "https");
    assert_eq!(parsed[1], "user");
    assert_eq!(parsed[2], "pass");
    assert_eq!(parsed[3], "1");
    assert_eq!(parsed[4], "example.com");
    assert_eq!(parsed[5], "8443");
    assert_eq!(parsed[6], "/b");
    assert_eq!(parsed[7], "x=1&x=2");
    assert_eq!(parsed[8], "1");
    assert_eq!(parsed[9], "frag");
    assert_eq!(parsed[10], "1");

    let built = sifr_stdlib::url::url_build_parts(
        "http",
        "::1",
        "/ready",
        None,
        Some(SifrIntBridge::from(8080)),
    )
    .unwrap_or_else(|err| panic!("build IPv6 URL: {err}"));
    assert_eq!(built[4], "[::1]");
    assert_eq!(built[5], "8080");
    assert_eq!(built[6], "/ready");

    assert_eq!(
        sifr_stdlib::url::url_percent_encode("a b/ç"),
        "a%20b%2F%C3%A7"
    );
    assert_eq!(
        sifr_stdlib::url::url_percent_decode("a%20b%2F%C3%A7")
            .unwrap_or_else(|err| panic!("decode text: {err}")),
        "a b/ç"
    );
    assert_eq!(
        sifr_stdlib::url::url_percent_encode_bytes(b"\xff /"),
        "%FF%20%2F"
    );
    assert_eq!(
        sifr_stdlib::url::url_percent_decode_bytes("%FF%20%2F")
            .unwrap_or_else(|err| panic!("decode bytes: {err}")),
        b"\xff /".to_vec()
    );
    assert_eq!(
        sifr_stdlib::url::url_normalize_path("/a/b/../c/./")
            .unwrap_or_else(|err| panic!("normalize path: {err}")),
        "/a/c/"
    );

    let flat = sifr_stdlib::url::url_query_parse_flat("a=1&b=two+words&a=3")
        .unwrap_or_else(|err| panic!("parse query: {err}"));
    assert_eq!(flat, vec!["a", "1", "b", "two words", "a", "3"]);
    assert_eq!(
        sifr_stdlib::url::url_query_build_flat(&flat)
            .unwrap_or_else(|err| panic!("build query: {err}")),
        "a=1&b=two+words&a=3"
    );

    assert!(sifr_stdlib::url::url_percent_decode("%zz").is_err());
    assert!(sifr_stdlib::url::url_parse_parts("https://é.example/").is_err());
    assert!(sifr_stdlib::url::url_build_parts(
        "https",
        "example.com",
        "/",
        None,
        Some(SifrIntBridge::from(70_000)),
    )
    .is_err());
    assert_eq!(sifr_stdlib::url::feature_name(), "url");
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

#[cfg(feature = "calendar")]
#[test]
fn calendar_leaf_matches_gregorian_helpers() {
    use sifr_runtime::interop::SifrIntBridge;

    let int = SifrIntBridge::from;

    assert!(sifr_stdlib::calendar::calendar_isleap(int(2000)));
    assert!(!sifr_stdlib::calendar::calendar_isleap(int(1900)));
    assert_eq!(
        sifr_stdlib::calendar::calendar_weekday(int(2024), int(2), int(29)),
        int(3)
    );
    assert_eq!(
        sifr_stdlib::calendar::calendar_monthrange(int(2024), int(2)),
        [int(3), int(29)]
    );
    assert_eq!(
        sifr_stdlib::calendar::calendar_monthrange(int(2023), int(2)),
        [int(2), int(28)]
    );
    assert_eq!(
        sifr_stdlib::calendar::calendar_monthrange(int(2024), int(13)),
        [int(2), int(30)]
    );
    let min_weekday =
        sifr_stdlib::calendar::calendar_weekday(int(i64::MIN), int(1), int(1)).to_i64_saturating();
    let max_weekday = sifr_stdlib::calendar::calendar_weekday(int(i64::MAX), int(12), int(31))
        .to_i64_saturating();
    assert!((0..=6).contains(&min_weekday));
    assert!((0..=6).contains(&max_weekday));
    assert_eq!(sifr_stdlib::calendar::feature_name(), "calendar");
}

#[cfg(feature = "uuid")]
#[test]
fn uuid_leaf_matches_public_uuid_helpers() {
    let generated = sifr_stdlib::uuid::uuid4();
    assert_eq!(generated.len(), 36);
    assert_eq!(generated.as_bytes()[14], b'4');
    assert!(matches!(
        generated.as_bytes()[19],
        b'8' | b'9' | b'a' | b'b'
    ));
    assert_eq!(
        sifr_stdlib::uuid::uuid3_text("6ba7b810-9dad-11d1-80b4-00c04fd430c8", "python.org"),
        "6fa459ea-ee8a-3ca4-894e-db77e160355e"
    );
    assert_eq!(
        sifr_stdlib::uuid::uuid5_text("6ba7b810-9dad-11d1-80b4-00c04fd430c8", "python.org"),
        "886313e1-3b8a-5372-9b90-0c9aee199e5d"
    );
    assert_eq!(
        sifr_stdlib::uuid::uuid3_text("not-a-uuid", "python.org"),
        "0421fac3-a9c6-3ea3-aee8-8f20aff3f278"
    );
    assert_eq!(sifr_stdlib::uuid::feature_name(), "uuid");
}

#[cfg(feature = "math")]
#[test]
fn math_leaf_matches_public_math_helpers() {
    use sifr_runtime::interop::SifrIntBridge;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 0.000_000_1,
            "expected {actual} to be close to {expected}"
        );
    }

    assert_close(sifr_stdlib::math::sqrt(9.0), 3.0);
    assert_close(sifr_stdlib::math::pow_val(2.0, 5.0), 32.0);
    assert_close(sifr_stdlib::math::dist(vec![0.0, 0.0], vec![3.0, 4.0]), 5.0);
    assert_close(
        sifr_stdlib::math::sumprod(vec![1.5, 2.0], vec![2.0, 3.0]),
        9.0,
    );
    assert_close(sifr_stdlib::math::frexp(8.0)[0], 0.5);
    assert_close(sifr_stdlib::math::modf(-1.25)[0], -0.25);
    assert_close(sifr_stdlib::math::gamma(5.0), 24.0);
    assert_close(sifr_stdlib::math::lgamma(5.0).exp(), 24.0);
    assert_eq!(sifr_stdlib::math::ulp(0.0), f64::from_bits(1));
    assert!(sifr_stdlib::math::ulp(f64::MAX).is_finite());
    assert!(sifr_stdlib::math::isnan(f64::NAN));
    assert!(sifr_stdlib::math::isinf(f64::INFINITY));
    assert_eq!(sifr_stdlib::math::floor(3.9), SifrIntBridge::from(3));
    assert_eq!(sifr_stdlib::math::ceil(3.1), SifrIntBridge::from(4));
    assert_eq!(sifr_stdlib::math::round_val(2.6), SifrIntBridge::from(3));
    assert_eq!(
        sifr_stdlib::math::isqrt(SifrIntBridge::from(10)),
        SifrIntBridge::from(3)
    );
    assert_eq!(sifr_stdlib::math::feature_name(), "math");
}

#[cfg(feature = "hash")]
#[test]
fn hash_leaf_matches_known_digest_vectors() {
    assert_eq!(
        sifr_stdlib::hash::sha256("abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        sifr_stdlib::hash::md5("abc"),
        "900150983cd24fb0d6963f7d28e17f72"
    );
    assert_eq!(
        sifr_stdlib::hash::sha1("abc"),
        "a9993e364706816aba3e25717850c26c9cd0d89d"
    );
    assert_eq!(sifr_stdlib::hash::sha224_bytes(b"abc").len(), 28);
    assert_eq!(sifr_stdlib::hash::sha384_bytes(b"abc").len(), 48);
    assert_eq!(sifr_stdlib::hash::sha512_bytes(b"abc").len(), 64);
    assert_eq!(sifr_stdlib::hash::blake2b_bytes(b"abc").len(), 64);
    assert_eq!(sifr_stdlib::hash::blake2s_bytes(b"abc").len(), 32);
    assert_eq!(sifr_stdlib::hash::feature_name(), "hash");
}

#[cfg(feature = "base64")]
#[test]
fn base64_leaf_matches_rfc_vectors_and_error_paths() {
    fn wrapcol(value: i64) -> sifr_runtime::interop::SifrIntBridge {
        sifr_runtime::interop::SifrIntBridge::from(value)
    }

    assert_eq!(sifr_stdlib::base64::base64_encode(""), "");
    assert_eq!(sifr_stdlib::base64::base64_encode("f"), "Zg==");
    assert_eq!(sifr_stdlib::base64::base64_encode("fo"), "Zm8=");
    assert_eq!(sifr_stdlib::base64::base64_encode("foo"), "Zm9v");
    assert_eq!(sifr_stdlib::base64::base64_encode("foobar"), "Zm9vYmFy");
    assert_eq!(
        sifr_stdlib::base64::base64_decode("Zm9v").expect("decode foo"),
        "foo"
    );
    assert_eq!(
        sifr_stdlib::base64::base64_encode_bytes(b"foo"),
        b"Zm9v".to_vec()
    );
    assert_eq!(
        sifr_stdlib::base64::base64_decode_bytes(b"Zm9v").expect("decode bytes"),
        b"foo".to_vec()
    );
    assert!(sifr_stdlib::base64::base64_decode("@@@@").is_err());
    assert_eq!(
        sifr_stdlib::base64::base64_encode_opts("foo", "-_", wrapcol(0)).expect("alt encode"),
        "Zm9v"
    );
    assert_eq!(
        sifr_stdlib::base64::base64_encode_opts("foobar", "", wrapcol(4)).expect("wrapped"),
        "Zm9v\nYmFy"
    );
    assert!(sifr_stdlib::base64::base64_encode_opts("x", "+", wrapcol(0)).is_err());
    assert!(sifr_stdlib::base64::base64_encode_opts("x", "", wrapcol(-1)).is_err());
    assert!(sifr_stdlib::base64::base64_decode_opts("YWJj!", "", true, "").is_err());
    assert_eq!(
        sifr_stdlib::base64::base64_decode_opts("Y W\nJj!", "", false, " \n!")
            .expect("ignore decode"),
        "abc"
    );
    assert_eq!(sifr_stdlib::base64::urlsafe_b64encode("hello"), "aGVsbG8=");
    assert_eq!(
        sifr_stdlib::base64::urlsafe_b64decode("aGVsbG8=").expect("urlsafe decode"),
        "hello"
    );
    assert_eq!(sifr_stdlib::base64::b32encode("foo"), "MZXW6===");
    assert_eq!(
        sifr_stdlib::base64::b32decode("mzxw6===").expect("base32 casefold"),
        "foo"
    );
    assert_eq!(sifr_stdlib::base64::b32hexencode("foo"), "CPNMU===");
    assert_eq!(
        sifr_stdlib::base64::b32hexdecode("cpnmu===").expect("base32hex casefold"),
        "foo"
    );
    assert!(sifr_stdlib::base64::b32decode("@").is_err());
    assert_eq!(sifr_stdlib::base64::feature_name(), "base64");
}

#[cfg(feature = "regex")]
#[test]
fn regex_leaf_matches_public_re_helpers() {
    use sifr_runtime::interop::SifrIntBridge;

    assert!(sifr_stdlib::regex::re_match("[0-9]+", "abc123").expect("match"));
    assert!(!sifr_stdlib::regex::re_match("[0-9]+", "abc").expect("no match"));
    assert_eq!(
        sifr_stdlib::regex::re_find("[0-9]+", "abc123def").expect("find"),
        Some("123".to_string())
    );
    assert_eq!(
        sifr_stdlib::regex::re_replace("[0-9]+", "#", "abc123def").expect("replace"),
        "abc#def"
    );
    assert_eq!(
        sifr_stdlib::regex::re_findall("[0-9]+", "x1y22z333").expect("findall"),
        vec!["1".to_string(), "22".to_string(), "333".to_string()]
    );
    assert_eq!(
        sifr_stdlib::regex::re_split(",", "a,b,c").expect("split"),
        vec!["a".to_string(), "b".to_string(), "c".to_string()]
    );
    assert_eq!(
        sifr_stdlib::regex::re_find_start("[0-9]+", "abc123")
            .expect("start")
            .to_i64_saturating(),
        3
    );
    assert_eq!(
        sifr_stdlib::regex::re_find_end("[0-9]+", "abc123")
            .expect("end")
            .to_i64_saturating(),
        6
    );
    assert!(
        sifr_stdlib::regex::re_match_flags("abc", "ABC", SifrIntBridge::from(2),)
            .expect("ignorecase")
    );
    assert!(sifr_stdlib::regex::re_match("(", "abc").is_err());
    assert_eq!(sifr_stdlib::regex::feature_name(), "regex");
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
    feature = "calendar",
    feature = "fs",
    feature = "gzip",
    feature = "hash",
    feature = "html",
    feature = "math",
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
        sifr_stdlib::calendar::feature_name(),
        sifr_stdlib::fs::feature_name(),
        sifr_stdlib::gzip::feature_name(),
        sifr_stdlib::hash::feature_name(),
        sifr_stdlib::html::feature_name(),
        sifr_stdlib::math::feature_name(),
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

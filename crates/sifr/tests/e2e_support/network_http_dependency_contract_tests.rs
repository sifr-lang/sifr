use super::*;

#[test]
pub(crate) fn test_generate_cargo_toml_url_module_uses_locked_parser_specs() {
    let stdlib_modules = normalize_dependency_set(vec!["sifr.url".to_string()].into_iter());
    let cargo_toml = generate_cargo_toml(&stdlib_modules, &BTreeSet::new(), "sifr_output");

    assert!(cargo_toml.contains("url = \"2.5.8\""));
    assert!(cargo_toml.contains("percent-encoding = \"2.3.2\""));
    assert!(!cargo_toml.contains("cookie = "));
    assert!(!cargo_toml.contains("http = "));
}

#[test]
pub(crate) fn test_generate_cargo_toml_http_module_uses_locked_header_specs_without_cookie_crate() {
    let stdlib_modules = normalize_dependency_set(vec!["sifr.http".to_string()].into_iter());
    let cargo_toml = generate_cargo_toml(&stdlib_modules, &BTreeSet::new(), "sifr_output");

    assert!(cargo_toml.contains("http = \"1.4.1\""));
    assert!(!cargo_toml.contains("cookie = "));
    assert!(!cargo_toml.contains("url = "));
    assert!(!cargo_toml.contains("percent-encoding"));
}

#[test]
pub(crate) fn test_generate_cargo_toml_url_http_required_crates_use_locked_specs() {
    let required_crates = normalize_dependency_set(
        vec![
            "url".to_string(),
            "percent-encoding".to_string(),
            "http".to_string(),
        ]
        .into_iter(),
    );
    let cargo_toml = generate_cargo_toml(&BTreeSet::new(), &required_crates, "sifr_output");

    assert!(cargo_toml.contains("url = \"2.5.8\""));
    assert!(cargo_toml.contains("percent-encoding = \"2.3.2\""));
    assert!(cargo_toml.contains("http = \"1.4.1\""));
    assert!(!cargo_toml.contains("cookie = "));
}

#[test]
pub(crate) fn test_generate_cargo_toml_http_transport_required_crates_use_locked_specs() {
    let required_crates = normalize_dependency_set(
        vec![
            "sifr_runtime".to_string(),
            "tokio".to_string(),
            "bytes".to_string(),
            "h2".to_string(),
            "http".to_string(),
            "http-body".to_string(),
            "http-body-util".to_string(),
            "hyper".to_string(),
            "hyper-util".to_string(),
            "tower-service".to_string(),
            "tokio-rustls".to_string(),
            "rustls".to_string(),
            "rustls-pemfile".to_string(),
            "rustls-platform-verifier".to_string(),
        ]
        .into_iter(),
    );
    let cargo_toml = generate_cargo_toml(&BTreeSet::new(), &required_crates, "sifr_output");

    assert!(
        cargo_toml.contains("sifr_runtime = ")
            && cargo_toml.contains("features = [\"net\", \"tls\", \"http\"]")
    );
    assert!(cargo_toml.contains("bytes = \"1.11.1\""));
    assert!(cargo_toml.contains("h2 = \"0.4.14\""));
    assert!(cargo_toml.contains("http-body = \"1.0.1\""));
    assert!(
        cargo_toml.contains("http-body-util = { version = \"0.1.3\", default-features = false }")
    );
    assert!(cargo_toml.contains(
        "hyper = { version = \"1.10.1\", default-features = false, features = [\"client\", \"http1\", \"http2\", \"server\"] }"
    ));
    assert!(cargo_toml.contains(
        "hyper-util = { version = \"0.1.20\", default-features = false, features = [\"tokio\"] }"
    ));
    assert!(cargo_toml.contains("tower-service = \"0.3.3\""));
}

#[test]
pub(crate) fn test_infer_dependencies_recognizes_url_http_runtime_references() {
    let rust_source = r#"
        let _url = url::Url::parse("https://example.com").unwrap();
        let _encoded = percent_encoding::percent_encode(
            b"x",
            percent_encoding::NON_ALPHANUMERIC,
        );
        let _header = http::HeaderName::from_static("content-type");
    "#;

    let (_, inferred_crates) = infer_dependencies(rust_source, &BTreeSet::new(), &BTreeSet::new());

    assert!(inferred_crates.contains("url"));
    assert!(inferred_crates.contains("percent-encoding"));
    assert!(inferred_crates.contains("http"));
    assert!(!inferred_crates.contains("cookie"));
}

#[test]
pub(crate) fn test_infer_dependencies_recognizes_http_transport_runtime_references() {
    let rust_source = r#"
        let _bytes = bytes::Bytes::new();
        let _h2 = h2::Reason::NO_ERROR;
        let _trailers: Option<http_body::Frame<bytes::Bytes>> = None;
        let _full = http_body_util::Full::new(bytes::Bytes::new());
        let _service = hyper::service::service_fn(|_| async {
            Ok::<_, std::convert::Infallible>(())
        });
        let _io = hyper_util::rt::TokioExecutor::new();
        let _svc: Option<Box<dyn tower_service::Service<(), Response = ()>>> = None;
    "#;

    let (_, inferred_crates) = infer_dependencies(rust_source, &BTreeSet::new(), &BTreeSet::new());

    assert!(inferred_crates.contains("bytes"));
    assert!(inferred_crates.contains("h2"));
    assert!(inferred_crates.contains("http-body"));
    assert!(inferred_crates.contains("http-body-util"));
    assert!(inferred_crates.contains("hyper"));
    assert!(inferred_crates.contains("hyper-util"));
    assert!(inferred_crates.contains("tower-service"));
}

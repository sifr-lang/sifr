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
pub(crate) fn test_infer_dependencies_recognizes_sysroot_net_references() {
    let rust_source = r#"
        async fn call_net() {
            let _stream = sifr_stdlib::net::net_connect_tcp("127.0.0.1:0", 1.0, true, "", false).await;
        }
    "#;

    let (stdlib_modules, _inferred_crates) =
        infer_dependencies(rust_source, &BTreeSet::new(), &BTreeSet::new());
    let cargo_toml = generate_cargo_toml(&stdlib_modules, &BTreeSet::new(), "sifr_output");

    assert!(stdlib_modules.contains("_sifr.net"));
    assert!(cargo_toml.contains("sifr_stdlib = { path = "));
    assert!(cargo_toml.contains("\"net\""));
}

#[test]
pub(crate) fn test_infer_dependencies_recognizes_http_runtime_crate_references() {
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

use super::*;

const BACKEND_EVIDENCE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/ecosystem_backend_certification/positive/backend_probe_coverage.sifr"
);
const BACKEND_NEGATIVE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/ecosystem_backend_certification/negative/sqlx_without_offline_artifacts.sifr"
);
const SQLX_QUERY_FILE: &str =
    ".sqlx/query-f2d6fe08dd19c716c98c45307c0649a03c0bf6d52c5d16c2375913d7a0f2f508.json";
const BACKEND_MODULE_SOURCE: &str = "src/bridges/mod.rs";
const BACKEND_IMPLEMENTATION_SOURCE: &str = "src/bridges/backend.rs";
const CFG_GATED_QUERY_SOURCE: &str = "src/bridges/cfg_gated_sqlx_regression.rs";
const INLINE_PATH_COMPILED_SOURCE: &str = "src/bridges/alt_inline/child.rs";
const INLINE_PATH_DEFAULT_SOURCE: &str = "src/bridges/inline_redirected/child.rs";

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-cargo-probe"]
fn test_build_backend_loopback_and_sqlx_offline_metadata() {
    let package_root = copied_backend_scenario("rust_interop_backend_positive");
    assert_exact_backend_dependency_graph(&package_root);
    install_evidence_source(
        &package_root,
        &format!(
            "{BACKEND_EVIDENCE}\n\ndef main() -> None:\n    print(verify_backend_probe_coverage())\n"
        ),
    );
    let entrypoint = package_entrypoint_from_cargo_layout(&package_root, "backend-feature-package");
    let errors = check_package_project(&entrypoint);
    assert!(
        errors.is_empty(),
        "bridge-safe backend package must pass compiler checking: {errors:#?}"
    );

    let output = built_package_output(&entrypoint);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(
            "axum=0.8.9;loopback=127.0.0.1:ephemeral;status=200;tower-http=0.7.0;middleware=response-header;sqlx=0.8.6;offline=true;query-value=13;query-hash=f2d6fe08dd19c716c98c45307c0649a03c0bf6d52c5d16c2375913d7a0f2f508;shutdown=clean"
        ),
        "real backend execution marker must be observed: {stdout}"
    );
    assert!(
        output.stderr.is_empty(),
        "backend evidence must not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-compiler-diagnostic"]
fn test_check_missing_and_stale_sqlx_offline_metadata_rejected_before_network() {
    let package_root = copied_backend_scenario("rust_interop_backend_sqlx_negative");
    install_cfg_gated_query_regression(&package_root);
    let listener = configure_dotenv_database_sentinel(&package_root);
    install_evidence_source(&package_root, BACKEND_NEGATIVE);
    let entrypoint = package_entrypoint_from_cargo_layout(&package_root, "backend-feature-package");
    let query_path = package_root.join(SQLX_QUERY_FILE);
    let metadata =
        std::fs::read_to_string(&query_path).expect("SQLx query cache should be readable");

    let control = check_package_project(&entrypoint);
    assert!(
        control.is_empty(),
        "checked-in SQLx offline metadata must be accepted before mutations: {control:#?}"
    );
    assert_database_sentinel_unused(&listener);

    std::fs::remove_file(&query_path).expect("SQLx query cache should be removed");
    assert_sqlx_metadata_mutation_is_rejected(&entrypoint, SqlxMetadataMutation::Missing);
    assert_database_sentinel_unused(&listener);

    let stale = metadata.replacen("SELECT 13::INT4 AS value", "SELECT 12::INT4 AS value", 1);
    assert_ne!(stale, metadata, "SQLx query mutation must change metadata");
    std::fs::write(&query_path, stale).expect("stale SQLx query cache should be written");
    assert_sqlx_metadata_mutation_is_rejected(&entrypoint, SqlxMetadataMutation::Stale);
    assert_database_sentinel_unused(&listener);
    let _ = std::fs::remove_dir_all(package_root);
}

#[derive(Clone, Copy)]
enum SqlxMetadataMutation {
    Missing,
    Stale,
}

fn assert_sqlx_metadata_mutation_is_rejected(
    entrypoint: &PackageEntrypoint,
    mutation: SqlxMetadataMutation,
) {
    let suffix = match mutation {
        SqlxMetadataMutation::Missing => "missing",
        SqlxMetadataMutation::Stale => "stale",
    };
    let errors = check_package_project(entrypoint);
    let rendered = format!("{errors:#?}");
    assert!(
        errors.iter().any(|error| {
            error.code == DiagnosticCode::RUST_CARGO_METADATA.code()
                && error
                    .message
                    .contains("Rust bridge package SQLx offline metadata")
        }),
        "{suffix} SQLx metadata must produce a stable Cargo diagnostic: {errors:#?}"
    );
    let expected_detail = match mutation {
        SqlxMetadataMutation::Missing => "there is no cached data for this query",
        SqlxMetadataMutation::Stale => "saved SQLx query text does not match query identity",
    };
    assert!(
        rendered.contains(expected_detail)
            && errors
                .iter()
                .any(|error| error.message.contains("SQLx offline metadata")),
        "{suffix} SQLx diagnostic must preserve the offline failure cause: {errors:#?}"
    );
    if matches!(mutation, SqlxMetadataMutation::Missing) {
        assert!(
            rendered.contains("SQLX_OFFLINE=true"),
            "missing metadata must prove SQLx offline mode was forced: {errors:#?}"
        );
    }
}

fn copied_backend_scenario(test_name: &str) -> PathBuf {
    let package_root = copied_scenario(
        "ecosystem_backend_certification",
        "backend_feature_package",
        test_name,
    );
    rebase_sifr_runtime_dependency(&package_root);
    package_root
}

fn configure_dotenv_database_sentinel(package_root: &Path) -> std::net::TcpListener {
    let listener =
        std::net::TcpListener::bind("127.0.0.1:0").expect("database sentinel should bind");
    listener
        .set_nonblocking(true)
        .expect("database sentinel should be nonblocking");
    let address = listener
        .local_addr()
        .expect("database sentinel address should resolve");
    std::fs::write(
        package_root.join(".env"),
        format!("DATABASE_URL=postgres://sifr:sifr@{address}/sifr\n"),
    )
    .expect("database sentinel dotenv should be installed");
    listener
}

fn install_cfg_gated_query_regression(package_root: &Path) {
    let source_path = package_root.join(BACKEND_MODULE_SOURCE);
    let mut source =
        std::fs::read_to_string(&source_path).expect("backend module source should be readable");
    source.push_str(
        r#"

#[cfg(test)]
mod cfg_gated_sqlx_regression;

#[path = "alt_inline"]
mod inline_redirected {
    mod child;
}
"#,
    );
    std::fs::write(source_path, source).expect("cfg-gated SQLx module should be declared");
    std::fs::write(
        package_root.join(CFG_GATED_QUERY_SOURCE),
        r#"
#[test]
fn query_without_offline_metadata() {
    let _ = sqlx::query!("SELECT 99::INT4 AS value");
}
"#,
    )
    .expect("cfg-gated SQLx regression source should be installed");
    for relative_path in [INLINE_PATH_COMPILED_SOURCE, INLINE_PATH_DEFAULT_SOURCE] {
        std::fs::create_dir_all(
            package_root
                .join(relative_path)
                .parent()
                .expect("inline module source should have a parent"),
        )
        .expect("inline module source directory should be installed");
    }
    std::fs::write(
        package_root.join(INLINE_PATH_COMPILED_SOURCE),
        "pub fn compiled_child() {}\n",
    )
    .expect("redirected inline module child should be installed");
    std::fs::write(
        package_root.join(INLINE_PATH_DEFAULT_SOURCE),
        "fn query_never_compiled() { let _ = sqlx::query!(\"SELECT 98::INT4 AS value\"); }\n",
    )
    .expect("never-compiled inline module sibling should be installed");

    let backend_path = package_root.join(BACKEND_IMPLEMENTATION_SOURCE);
    let mut backend =
        std::fs::read_to_string(&backend_path).expect("backend source should be readable");
    backend.push_str(
        r#"

#[path = "direct_path_regression.rs"]
mod direct_path_regression;

#[path = "file_inline_alt"]
mod file_inline_redirected {
    mod child;
}

#[path = "loaded_path_regression.rs"]
mod loaded_path_regression;
"#,
    );
    std::fs::write(backend_path, backend).expect("file-module path regressions should be declared");
    for (relative_path, source) in [
        (
            "src/bridges/direct_path_regression.rs",
            "pub fn compiled_direct() {}\n",
        ),
        (
            "src/bridges/backend/direct_path_regression.rs",
            "fn query_never_compiled() { let _ = sqlx::query!(\"SELECT 97::INT4 AS value\"); }\n",
        ),
        (
            "src/bridges/file_inline_alt/child.rs",
            "pub fn compiled_inline_child() {}\n",
        ),
        (
            "src/bridges/backend/file_inline_alt/child.rs",
            "fn query_never_compiled() { let _ = sqlx::query!(\"SELECT 96::INT4 AS value\"); }\n",
        ),
        (
            "src/bridges/loaded_path_regression.rs",
            "mod loaded_path_child;\n",
        ),
        (
            "src/bridges/loaded_path_child.rs",
            "pub fn compiled_loaded_child() {}\n",
        ),
        (
            "src/bridges/loaded_path_regression/loaded_path_child.rs",
            "fn query_never_compiled() { let _ = sqlx::query!(\"SELECT 95::INT4 AS value\"); }\n",
        ),
    ] {
        let source_path = package_root.join(relative_path);
        std::fs::create_dir_all(
            source_path
                .parent()
                .expect("path regression source should have a parent"),
        )
        .expect("path regression source directory should be installed");
        std::fs::write(source_path, source).expect("path regression source should be installed");
    }
}

fn assert_database_sentinel_unused(listener: &std::net::TcpListener) {
    match listener.accept() {
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
        Ok((_stream, address)) => {
            panic!("SQLx offline validation unexpectedly connected from {address}")
        }
        Err(error) => panic!("database sentinel accept failed: {error}"),
    }
}

fn assert_exact_backend_dependency_graph(package_root: &Path) {
    let output = std::process::Command::new("cargo")
        .args([
            "tree",
            "--workspace",
            "--edges",
            "features",
            "--locked",
            "--offline",
        ])
        .current_dir(package_root)
        .output()
        .expect("locked backend Cargo tree should execute");
    assert!(
        output.status.success(),
        "locked backend Cargo tree must pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let tree = String::from_utf8_lossy(&output.stdout);
    for package in [
        "axum v0.8.9",
        "tower-http v0.7.0",
        "tower-http feature \"set-header\"",
        "sqlx v0.8.6",
        "sqlx feature \"runtime-tokio-rustls\"",
        "sqlx feature \"postgres\"",
        "sqlx feature \"macros\"",
    ] {
        assert!(
            tree.contains(package),
            "locked graph must contain {package}: {tree}"
        );
    }
}

//! Complete frontend differential for the semantic/native ownership boundary.
#[test]
#[ignore = "explicit complete semantic/native corpus qualification"]
fn semantic_native_full_frontend_corpus() {
    let compiler = crate::CompilerContext::for_test();
    let semantic = sifr_semantic_services::SemanticContext::with_sysroot(
        compiler.identity().clone(),
        compiler.sysroot().unwrap().clone(),
    );
    let native = crate::stdlib::compile_stdlib_uncached().unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../sifr/tests/e2e");
    let mut failures = Vec::new();
    let mut outcomes = [0; 3];
    for (index, directory) in ["pass", "fail", "runtime_fail"].iter().enumerate() {
        let mut paths = std::fs::read_dir(root.join(directory))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "sifr")
            })
            .collect::<Vec<_>>();
        paths.sort();
        assert!(!paths.is_empty(), "empty {directory} inventory");
        for path in paths {
            let source = std::fs::read_to_string(&path).unwrap();
            let expected = crate::lower_source(&compiler, &source);
            let actual = sifr_semantic_services::single_file::lower_source(&semantic, &source);
            let difference = match (expected, actual) {
                (Ok(expected), Ok(actual)) => {
                    let expected_sql = sifr_frontend::sql_migration_declarations(&expected.module);
                    let actual_sql = sifr_frontend::sql_migration_declarations(&actual.module);
                    let expected_rust = sifr_codegen::generate_rust_with_stdlib_for_module(
                        &expected.module,
                        &native.code,
                        Some("main"),
                    );
                    let actual_rust = sifr_codegen::generate_rust_with_stdlib_for_module(
                        &actual.module,
                        &native.code,
                        Some("main"),
                    );
                    if expected_sql != actual_sql {
                        Some("migration projection differs".to_string())
                    } else if expected_rust.rust_source != actual_rust.rust_source {
                        Some("generated Rust differs".to_string())
                    } else {
                        None
                    }
                }
                (Err(expected), Err(actual)) => {
                    let expected = serde_json::to_value(expected).unwrap();
                    let actual = serde_json::to_value(actual).unwrap();
                    (expected != actual)
                        .then(|| format!("diagnostics differ: expected={expected} actual={actual}"))
                }
                (Ok(_), Err(actual)) => {
                    Some(format!("semantic rejected native success: {actual:?}"))
                }
                (Err(expected), Ok(_)) => {
                    Some(format!("semantic accepted native failure: {expected:?}"))
                }
            };
            if let Some(difference) = difference {
                failures.push(format!("{}: {difference}", path.display()));
            }
            outcomes[index] += 1;
        }
    }
    assert!(
        outcomes[0] >= 732 && outcomes[1] >= 569 && outcomes[2] >= 6,
        "inventory shrank: {outcomes:?}"
    );
    assert!(
        failures.is_empty(),
        "{} frontend differences:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
#[test]
fn semantic_native_migration_projection_and_error_parity() {
    let compiler = crate::CompilerContext::for_test();
    let semantic = sifr_semantic_services::SemanticContext::with_sysroot(
        compiler.identity().clone(),
        compiler.sysroot().unwrap().clone(),
    );
    let source = include_str!(
        "../../../../verification/areas/sql_platform/fixtures/migration_source/add_status.sifr"
    );
    assert_eq!(
        crate::compile_sql_migration_source(&compiler, source).unwrap(),
        sifr_semantic_services::compile_sql_migration_source(&semantic, source).unwrap()
    );
    for source in [
        "def broken(:\n",
        "def main():\n    missing()\n",
        "from nowhere import missing\n",
    ] {
        assert_eq!(
            serde_json::to_value(
                crate::compile_sql_migration_source(&compiler, source).unwrap_err()
            )
            .unwrap(),
            serde_json::to_value(
                sifr_semantic_services::compile_sql_migration_source(&semantic, source)
                    .unwrap_err()
            )
            .unwrap()
        );
    }
}

use super::*;

#[test]
fn configured_profile_import_diagnostic_precedes_lowering_and_preserves_span() {
    let fixture = profile_fixture_with_components(
        fixture_component(&schema_response()),
        fixture_component(&query_response()),
    );
    let prepared = prepare_sql_profiles(&fixture.graph, &fixture.owner_id)
        .expect("offline schema profile preparation should succeed");
    let source = r#"@cache.query
def cached() -> int:
    return 1

@app.query
def find_user(user_id: int64) -> Template:
    return app.sql(t"SELECT {user_id} AS value")
"#;
    let diagnostics = super::super::sql_profiles::sql_profile_import_diagnostics(
        source,
        "src/main.sifr",
        &prepared,
    );
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic.code,
        sifr_diagnostics::DiagnosticCode::SQL_PROFILE_IMPORT.code()
    );
    assert!(diagnostic.message.contains("find_user"));
    assert_eq!(
        diagnostic.help.as_deref(),
        Some("Add 'from sifr.sql.schemas import app'.")
    );
    assert_eq!(diagnostic.spans.len(), 1);
    assert_eq!(diagnostic.spans[0].file.as_deref(), Some("src/main.sifr"));
    assert_eq!(diagnostic.spans[0].line, Some(5));
    assert_eq!(
        &source[diagnostic.spans[0].byte_start as usize..diagnostic.spans[0].byte_end as usize],
        "app.query"
    );
    let standalone = r#"def find_user(user_id: int64) -> Template:
    return app.sql(t"SELECT {user_id} AS value")
"#;
    let standalone_diagnostics = super::super::sql_profiles::sql_profile_import_diagnostics(
        standalone,
        "src/main.sifr",
        &prepared,
    );
    assert_eq!(standalone_diagnostics.len(), 1);
    assert_eq!(
        standalone_diagnostics[0].code,
        sifr_diagnostics::DiagnosticCode::SQL_PROFILE_IMPORT.code()
    );
    assert!(
        standalone_diagnostics[0]
            .message
            .contains("SQL constructor")
    );
}

#[test]
fn profile_import_discovery_respects_aliases_and_shadowing() {
    let fixture = profile_fixture_with_components(
        fixture_component(&schema_response()),
        fixture_component(&query_response()),
    );
    let prepared = prepare_sql_profiles(&fixture.graph, &fixture.owner_id)
        .expect("offline schema profile preparation should succeed");
    for source in [
        r#"from sifr.sql.schemas import app as database
@database.query
def query() -> Template:
    return database.sql(t"SELECT 1")
"#,
        r#"from cache import app
@app.query
def query() -> int:
    return 1
"#,
        r#"from sifr.sql.schemas import app
app = cache
@app.query
def query() -> int:
    return 1
"#,
        r#"@cache.query
def query() -> int:
    return 1
"#,
        r#"def query(app: Cache) -> Template:
    return app.sql(t"SELECT 1")
"#,
    ] {
        assert!(
            super::super::sql_profiles::sql_profile_import_diagnostics(
                source,
                "main.sifr",
                &prepared
            )
            .is_empty(),
            "{source}"
        );
    }
}

#[test]
fn package_check_reports_profile_import_before_undefined_name() {
    let fixture = profile_fixture_with_components(
        fixture_component(&schema_response()),
        fixture_component(&query_response()),
    );
    let main_file = fixture.owner_root.join("src/main.sifr");
    std::fs::create_dir_all(main_file.parent().expect("source directory"))
        .expect("create source directory");
    std::fs::write(
        &main_file,
        "@app.query
def find_user(user_id: int64) -> Template:
    return app.sql(t\"SELECT {user_id} AS value\")
",
    )
    .expect("write package source");
    for package in fixture.graph.packages.values() {
        std::fs::create_dir_all(package.package_root.join("src"))
            .expect("create package source root");
    }
    let source_map = sifr_package::PackageSourceMap::build(
        &fixture.graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("package source map");
    let entrypoint = crate::PackageEntrypoint {
        main_file,
        package_id: fixture.owner_id,
        graph: fixture.graph,
        source_map,
        python_runtime: None,
        lock_mode: sifr_package::CargoLockMode::Normal,
    };
    let errors = crate::check_package_project(
        &crate::CompilerContext::for_test(),
        &entrypoint,
        &mut sifr_frontend::DiskSourceProvider::new(),
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(
        errors[0].code,
        sifr_diagnostics::DiagnosticCode::SQL_PROFILE_IMPORT.code()
    );
}

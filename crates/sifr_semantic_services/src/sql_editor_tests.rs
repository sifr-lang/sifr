use super::*;
use sifr_package::{
    CargoPackageId, DirectDependencyScope, ImportRoot, ScopedImport, ScopedImportSource,
    SifrManifest, SifrPackageMetadata,
};
use std::path::{Path, PathBuf};

#[test]
fn missing_lock_defers_profiles() {
    let root = tempfile::tempdir().expect("temporary editor project");
    std::fs::write(
        root.path().join("Cargo.toml"),
        "[package]\nname = \"editor\"\nversion = \"0.1.0\"\n",
    )
    .expect("Cargo manifest");
    std::fs::write(root.path().join("sifr.toml"), "[source]\nroot = \"src\"\n")
        .expect("Sifr manifest");
    let entrypoint = root.path().join("src/main.sifr");
    let prepared = load_sql_editor_profiles(root.path(), &entrypoint)
        .expect("unlocked editor project must defer SQL profiles");
    assert!(prepared.registry().is_empty());
    assert!(prepared.requirements().is_empty());
    assert!(prepared.initialization_diagnostics().is_empty());
    assert!(!root.path().join("Cargo.lock").exists());
}

#[test]
fn invalid_component_preserves_source_diagnostic() {
    let root = tempfile::tempdir().expect("temporary profile graph");
    let owner_root = root.path().join("app");
    let provider_root = root.path().join("postgres");
    std::fs::create_dir_all(&owner_root).expect("owner");
    std::fs::create_dir_all(&provider_root).expect("provider");
    let owner = package(
        "app",
        &owner_root,
        r#"[package]
name = "app"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[sql.profiles.app]
provider = "postgres"
family = "postgresql"
source = "db/schema.sql"
server-version = "18"
pooling = "session"
schema-evidence = "migration-head"
schema-strictness = "compatible"
"#,
    );
    let owner_id = owner.package_id.clone();
    let provider = package(
        "postgres",
        &provider_root,
        "[package]\nname = \"sifr_sql_postgresql\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n",
    );
    let provider_id = provider.package_id.clone();
    let graph = SifrPackageGraph {
        packages: BTreeMap::from([(owner_id.clone(), owner.clone())]),
        cargo_edges: BTreeMap::new(),
        direct_dependency_scopes: BTreeMap::new(),
        backend_crates: BTreeMap::new(),
        classifications: BTreeMap::new(),
    };
    let missing_profile = prepare_sql_profiles(&graph, &owner_id)
        .expect_err("missing configured provider must retain package origin");
    assert!(!missing_profile.is_empty());
    assert!(missing_profile.iter().any(|diagnostic| {
        diagnostic
            .args
            .get("manifest_path")
            .is_some_and(|value| format!("{value:?}").contains("sifr.toml"))
    }));

    let graph = SifrPackageGraph {
        packages: BTreeMap::from([(owner_id.clone(), owner), (provider_id.clone(), provider)]),
        cargo_edges: BTreeMap::from([(owner_id.clone(), BTreeSet::from([provider_id.clone()]))]),
        direct_dependency_scopes: BTreeMap::from([(
            owner_id.clone(),
            DirectDependencyScope {
                imports: BTreeMap::from([(
                    ImportRoot("sifr_sql_postgresql".to_string()),
                    ScopedImport {
                        import_root: ImportRoot("sifr_sql_postgresql".to_string()),
                        target_export_root: ImportRoot("sifr_sql_postgresql".to_string()),
                        package_id: provider_id,
                        cargo_package_id: cargo_id("postgres"),
                        dependency_name: "postgres".to_string(),
                        source: ScopedImportSource::Export,
                    },
                )]),
            },
        )]),
        backend_crates: BTreeMap::new(),
        classifications: BTreeMap::new(),
    };
    let missing_component = prepare_sql_profiles(&graph, &owner_id)
        .expect_err("provider without schema component must fail preparation");
    assert_eq!(missing_component.len(), 1);
    assert_eq!(
        missing_component[0].code,
        DiagnosticCode::PACKAGE_MISSING_OR_INVALID_SIFR_MANIFEST.code(),
        "{missing_component:?}"
    );
    assert!(
        missing_component[0]
            .message
            .contains("no compiler component")
    );
    assert!(
        missing_component[0]
            .args
            .get("manifest_key")
            .is_some_and(|value| {
                matches!(value, DiagnosticArg::String(key) if key == "sql.profiles.app.provider")
            })
    );
}

#[test]
fn wrong_profile_import_names_exact_target() {
    let source = "from sifr.sql.schemas import other\n@cache.query\ndef cached() -> int:\n    return 1\n\n@app.query\ndef selected() -> Template:\n    return app.sql(t\"SELECT 1\")\n";
    let configured = BTreeSet::from(["app".to_string()]);
    let diagnostics =
        sql_profile_import_diagnostics_for_names(source, "src/main.sifr", &configured);
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.code, DiagnosticCode::SQL_PROFILE_IMPORT.code());
    assert!(diagnostic.message.contains("selected"));
    assert_eq!(diagnostic.spans.len(), 1);
    let span = &diagnostic.spans[0];
    assert_eq!(span.file.as_deref(), Some("src/main.sifr"));
    assert_eq!(
        &source[span.byte_start as usize..span.byte_end as usize],
        "app.query"
    );
    assert_eq!(
        diagnostic.help.as_deref(),
        Some("Add 'from sifr.sql.schemas import app'.")
    );
    let corrected = source.replace("import other", "import app");
    assert!(
        sql_profile_import_diagnostics_for_names(&corrected, "src/main.sifr", &configured)
            .is_empty()
    );
}

fn cargo_id(name: &str) -> CargoPackageId {
    CargoPackageId(format!("registry+https://example.invalid#{name}@1.0.0"))
}

fn package(name: &str, root: &Path, manifest_source: &str) -> SifrPackageMetadata {
    let manifest_path = root.join("sifr.toml");
    let manifest = SifrManifest::parse(&cargo_id(name), &manifest_path, manifest_source)
        .expect("fixture manifest");
    SifrPackageMetadata {
        package_id: SifrPackageId(format!("sifr-{name}@1.0.0#registry")),
        cargo_package_id: cargo_id(name),
        cargo_package_name: format!("sifr-{name}"),
        cargo_version: "1.0.0".to_string(),
        cargo_source: Some("registry+https://example.invalid/index".to_string()),
        package_root: PathBuf::from(root),
        sifr_manifest: manifest_path,
        sifr_name: manifest.package_name.clone(),
        manifest,
        aliases: BTreeMap::new(),
    }
}

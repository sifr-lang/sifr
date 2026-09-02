use crate::test_support::{TestExpectErr as _, TestUnwrap as _};

use crate::cargo::metadata::CargoPackageId;
use crate::graph::derive::{SifrPackageGraph, SifrPackageId, SifrPackageMetadata};
use crate::graph::scopes::{DirectDependencyScope, ScopedImport, ScopedImportSource};
use crate::{
    ImportRoot, SchemaSourceKind, SifrManifest, resolve_sql_profiles, resolve_sql_requirements,
};
use sifr_sql_contract::{
    DialectIdentity, SCHEMA_IR_FORMAT_VERSION, SchemaDocumentKind, SchemaIr, SchemaSourceInput,
    schema_source_fingerprint,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[test]
fn manifest_parses_complete_offline_profile_contract() {
    let manifest = parse(&app_manifest("sifr_sql_postgresql"));
    let profile = &manifest.sql.profiles["app"];
    assert_eq!(profile.sources, vec![PathBuf::from("db/schema.sql")]);
    assert_eq!(profile.source_kind, SchemaSourceKind::SqlDdl);
    assert_eq!(profile.server_version, "18");
    assert_eq!(profile.session.search_path, ["app", "public"]);
    assert_eq!(
        profile.session.sql_modes,
        BTreeSet::from(["standard".to_string()])
    );
    assert_eq!(profile.session.time_zone.as_deref(), Some("UTC"));
    let requirement = &manifest.sql.requirements["has_users"];
    assert_eq!(
        requirement.capabilities,
        BTreeSet::from([
            "sql.bind.parameters".to_string(),
            "sql.expression.equality".to_string(),
            "sql.query.select".to_string(),
        ])
    );
    assert_eq!(
        requirement.providers["postgresql"].source,
        PathBuf::from("db/requirements/has_users.postgresql.sql")
    );
}

#[test]
fn sqlite_profile_aliases_compile_flags_and_required_features_into_identity_inputs() {
    let source = app_manifest("sifr_sql_sqlite")
        .replace("server-version = \"18\"", "server-version = \"3.53.2\"")
        .replace(
            "search-path = [\"app\", \"public\"]",
            "search-path = [\"main\", \"analytics\"]",
        )
        .replace(
            "extensions = [\"citext\"]",
            "extensions = []\nrequired-features = [\"json\", \"fts5\"]",
        )
        .replace(
            "sql-modes = [\"standard\"]",
            "sql-modes = []\ncompile-flags = [\"ENABLE_FTS5\"]",
        );
    let manifest = parse(&source);
    let profile = &manifest.sql.profiles["app"];
    assert_eq!(profile.server_version, "3.53.2");
    assert_eq!(
        profile.extensions,
        BTreeSet::from(["fts5".to_string(), "json".to_string()])
    );
    assert_eq!(
        profile.session.sql_modes,
        BTreeSet::from(["ENABLE_FTS5".to_string()])
    );
}

#[test]
fn manifest_rejects_credentials_and_live_connection_inputs() {
    for forbidden in [
        "database-url = \"postgresql://secret@localhost/app\"",
        "password = \"secret\"",
        "credentials-env = \"DATABASE_URL\"",
    ] {
        let source = app_manifest("sifr_sql_postgresql").replace(
            "server-version = \"18\"",
            &format!("server-version = \"18\"\n{forbidden}"),
        );
        let error = SifrManifest::parse(&cargo_id("app"), Path::new("/ws/app/sifr.toml"), &source)
            .test_expect_err("credential-bearing profile input must fail");
        assert!(error.message.contains("unsupported field"));
        assert!(!error.message.contains("secret"));
    }
    let source = app_manifest("sifr_sql_postgresql").replace(
        "sql-modes = [\"standard\"]",
        "sql-modes = [\"postgresql://user:secret@host/db\"]",
    );
    let error = SifrManifest::parse(&cargo_id("app"), Path::new("/ws/app/sifr.toml"), &source)
        .test_expect_err("SQL modes must not carry arbitrary credential strings");
    assert!(error.message.contains("SQL mode identifiers"));
    assert!(!error.message.contains("secret"));
}

#[test]
fn reusable_library_can_declare_requirements_without_an_application_profile() {
    let manifest = parse(
        r#"[package]
name = "library"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[sql.requirements.has_users]
capabilities = ["sql.query.select"]

[sql.requirements.has_users.providers.postgresql]
provider = "postgres"
source = "db/requirements/has_users.postgresql.sql"
server-version = "13"
extensions = []
sql-modes = []
"#,
    );
    assert!(manifest.sql.profiles.is_empty());
    assert_eq!(manifest.sql.requirements.len(), 1);
}

#[test]
fn profile_provider_resolves_to_locked_package_and_component_identity() {
    let owner_manifest = app_manifest("postgres").replace(
        "time-zone = \"UTC\"",
        "time-zone = \"UTC\"\ncollation = \"legacy-collation\"\ncharacter-set = \"legacy-character-set\"",
    );
    let owner = package("app", parse(&owner_manifest));
    let provider = package("postgres", parse(&provider_manifest()));
    let owner_id = owner.package_id.clone();
    let provider_id = provider.package_id.clone();
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
    let resolved = resolve_sql_profiles(&graph, &owner_id).test_unwrap("resolve profiles");
    let profile = &resolved["app"];
    assert_eq!(profile.provider.package_version.to_string(), "1.0.0");
    assert_eq!(
        profile.provider.compiler_components["postgresql@1.0.0"],
        "a".repeat(64)
    );
    assert!(
        profile
            .provider
            .package_graph_digest
            .starts_with("fnv1a64:")
    );
    let mut wrong_provider = profile.provider.clone();
    wrong_provider.package_id = "different@1.0.0#registry".to_string();
    let wrong_schema = SchemaIr {
        format_version: SCHEMA_IR_FORMAT_VERSION,
        provider: wrong_provider,
        dialect: DialectIdentity {
            family: "postgresql".to_string(),
            server_version: "18".to_string(),
            modes: BTreeSet::new(),
            features: BTreeSet::from(["citext".to_string()]),
        },
        objects: BTreeMap::new(),
    };
    assert!(
        profile
            .build_authority(
                wrong_schema,
                &[],
                BTreeSet::from(["sql.query.select".to_string()]),
            )
            .is_err()
    );
    let source = SchemaSourceInput {
        document: "db/schema.sql".to_string(),
        kind: SchemaDocumentKind::SqlDdl,
        fingerprint: schema_source_fingerprint(b""),
        contents: Vec::new(),
    };
    let compatible_schema = SchemaIr {
        format_version: SCHEMA_IR_FORMAT_VERSION,
        provider: profile.provider.clone(),
        dialect: DialectIdentity {
            family: "postgresql".to_string(),
            server_version: "18".to_string(),
            modes: BTreeSet::from(["standard".to_string()]),
            features: BTreeSet::from(["citext".to_string()]),
        },
        objects: BTreeMap::new(),
    };
    assert!(
        profile
            .build_authority(
                compatible_schema,
                &[source],
                BTreeSet::from(["sql.query.select".to_string()]),
            )
            .is_ok()
    );
    let requirements =
        resolve_sql_requirements(&graph, &owner_id).test_unwrap("resolve schema requirements");
    let requirement = &requirements[&format!("{}::has_users", owner_id.0)];
    assert_eq!(
        requirement.providers["postgresql"].provider,
        profile.provider
    );
    assert_eq!(requirement.config.providers.len(), 1);
}

fn package(name: &str, manifest: SifrManifest) -> SifrPackageMetadata {
    SifrPackageMetadata {
        package_id: SifrPackageId(format!("sifr-{name}@1.0.0#registry")),
        cargo_package_id: cargo_id(name),
        cargo_package_name: format!("sifr-{name}"),
        cargo_version: "1.0.0".to_string(),
        cargo_source: Some("registry+https://github.com/rust-lang/crates.io-index".to_string()),
        package_root: PathBuf::from(format!("/ws/{name}")),
        sifr_manifest: PathBuf::from(format!("/ws/{name}/sifr.toml")),
        sifr_name: manifest.package_name.clone(),
        manifest,
        aliases: BTreeMap::new(),
    }
}

fn parse(source: &str) -> SifrManifest {
    SifrManifest::parse(&cargo_id("app"), Path::new("/ws/app/sifr.toml"), source)
        .test_unwrap("manifest")
}

fn cargo_id(name: &str) -> CargoPackageId {
    CargoPackageId(format!("registry+https://example.invalid#{name}@1.0.0"))
}

fn app_manifest(provider: &str) -> String {
    format!(
        r#"[package]
name = "app"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[sql.profiles.app]
provider = "{provider}"
family = "postgresql"
source = "db/schema.sql"
server-version = "18"
search-path = ["app", "public"]
extensions = ["citext"]
sql-modes = ["standard"]
pooling = "session"
schema-evidence = "migration-head"
schema-strictness = "compatible"

[sql.profiles.app.session]
time-zone = "UTC"

[sql.requirements.has_users]
capabilities = ["sql.bind.parameters", "sql.expression.equality", "sql.query.select"]

[sql.requirements.has_users.providers.postgresql]
provider = "{provider}"
source = "db/requirements/has_users.postgresql.sql"
server-version = "13"
extensions = []
sql-modes = []
"#
    )
}

fn provider_manifest() -> String {
    format!(
        r#"[package]
name = "sifr_sql_postgresql"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[compiler-components.postgresql]
kind = "embedded-language-provider"
artifact = "components/postgresql.wasm"
version = "1.0.0"
sha256 = "{}"
protocol-min = 1
protocol-max = 1
processors = ["sifr.sql.postgresql.schema", "sifr.sql.postgresql.sql"]
diagnostic-namespace = "SQL-POSTGRESQL"
diagnostics = [{{ code = "SIFR-SQL-POSTGRESQL-0001", lifecycle = "active" }}]
"#,
        "a".repeat(64)
    )
}

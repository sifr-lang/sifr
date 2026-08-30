use super::sql_profiles::prepare_sql_profiles;
use sha2::{Digest, Sha256};
use sifr_compiler_component::{
    ClosedType, EmbeddedAnalysisResponse, EmbeddedPlan, PlanKind, RuntimeLowering,
    SemanticOperation, compute_plan_fingerprint,
};
use sifr_package::{
    CargoPackageId, DirectDependencyScope, ImportRoot, ScopedImport, ScopedImportSource,
    SifrManifest, SifrPackageGraph, SifrPackageId, SifrPackageMetadata,
};
use sifr_sql_contract::{
    DialectIdentity, ObjectId, SCHEMA_NORMALIZATION_PAYLOAD_TAG, SchemaDocument,
    SchemaDocumentKind, SchemaNormalizationOutput, SchemaObject, SchemaObjectKind,
    SchemaSourceLocation,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[test]
fn package_compilation_prepares_profiles_offline_and_binds_source_bytes() {
    let fixture = profile_fixture(fixture_component(&schema_response()));
    let prepared = prepare_sql_profiles(&fixture.graph, &fixture.owner_id)
        .expect("offline schema profile preparation should succeed");
    assert!(
        prepared
            .module("app")
            .is_some_and(|module| module.module_path == "sifr.sql.schemas.app")
    );
    let first = prepared.cache_fragment();
    std::fs::write(
        fixture.owner_root.join("db/schema.sql"),
        "create schema public;\n-- semantic source change\n",
    )
    .expect("update checked-in source");
    let second = prepare_sql_profiles(&fixture.graph, &fixture.owner_id)
        .expect("changed checked-in source should still normalize")
        .cache_fragment();
    assert_ne!(first, second);
    assert!(first.contains("sifr.sql.schemas.app"));
}

#[test]
fn package_compilation_denies_ambient_schema_component_capabilities() {
    let bytes = wat::parse_str(
        r#"(component
            (type $clock (instance
                (type $now (func (result u64)))
                (export "now" (func (type $now)))))
            (import "wasi:clocks/wall-clock@0.2.0" (instance $clock (type $clock))))"#,
    )
    .expect("ambient import fixture");
    let fixture = profile_fixture(bytes);
    let errors = prepare_sql_profiles(&fixture.graph, &fixture.owner_id)
        .expect_err("ambient component capability must fail compilation");
    assert_eq!(
        errors[0].code,
        sifr_diagnostics::DiagnosticCode::COMPONENT_CAPABILITY.code()
    );
}

struct ProfileFixture {
    graph: SifrPackageGraph,
    owner_id: SifrPackageId,
    owner_root: PathBuf,
}

fn profile_fixture(component_bytes: Vec<u8>) -> ProfileFixture {
    let root = temp_root("sql-profile");
    let owner_root = root.join("app");
    let provider_root = root.join("postgres");
    std::fs::create_dir_all(owner_root.join("db")).expect("owner schema directory");
    std::fs::create_dir_all(provider_root.join("components"))
        .expect("provider component directory");
    std::fs::write(owner_root.join("db/schema.sql"), "create schema public;\n")
        .expect("schema source");
    std::fs::write(
        provider_root.join("components/postgresql.wasm"),
        &component_bytes,
    )
    .expect("component artifact");
    let component_sha = lower_hex(&Sha256::digest(&component_bytes));
    let owner = package("app", &owner_root, parse_manifest(&owner_manifest()));
    let provider = package(
        "postgres",
        &provider_root,
        parse_manifest(&provider_manifest(&component_sha)),
    );
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
    ProfileFixture {
        graph,
        owner_id,
        owner_root,
    }
}

fn package(name: &str, root: &Path, manifest: SifrManifest) -> SifrPackageMetadata {
    SifrPackageMetadata {
        package_id: SifrPackageId(format!("sifr-{name}@1.0.0#registry")),
        cargo_package_id: cargo_id(name),
        cargo_package_name: format!("sifr-{name}"),
        cargo_version: "1.0.0".to_string(),
        cargo_source: Some("registry+https://example.invalid/index".to_string()),
        package_root: root.to_path_buf(),
        sifr_manifest: root.join("sifr.toml"),
        sifr_name: manifest.package_name.clone(),
        manifest,
        aliases: BTreeMap::new(),
    }
}

fn parse_manifest(source: &str) -> SifrManifest {
    SifrManifest::parse(
        &cargo_id("fixture"),
        Path::new("/fixture/sifr.toml"),
        source,
    )
    .expect("fixture manifest")
}

fn owner_manifest() -> String {
    r#"[package]
name = "app"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[sql.profiles.app]
provider = "postgres"
source = "db/schema.sql"
server-version = "18"
search-path = ["public"]
extensions = ["citext"]
sql-modes = ["standard"]
pooling = "session"
schema-evidence = "migration-head"
schema-strictness = "compatible"
"#
    .to_string()
}

fn provider_manifest(sha256: &str) -> String {
    format!(
        r#"[package]
name = "sifr_sql_postgresql"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[compiler-components.postgresql]
kind = "embedded-language-provider"
artifact = "components/postgresql.wasm"
version = "1.0.0"
sha256 = "{sha256}"
protocol-min = 1
protocol-max = 1
processors = ["sifr.sql.postgresql.schema", "sifr.sql.postgresql.sql"]
diagnostic-namespace = "SQL-POSTGRESQL"
diagnostics = [{{ code = "SIFR-SQL-POSTGRESQL-0001", lifecycle = "active" }}]
"#
    )
}

fn schema_response() -> Vec<u8> {
    let output = SchemaNormalizationOutput {
        dialect: DialectIdentity {
            family: "postgresql".to_string(),
            server_version: "18".to_string(),
            modes: BTreeSet::from(["standard".to_string()]),
            features: BTreeSet::from(["citext".to_string()]),
        },
        documents: vec![SchemaDocument {
            kind: SchemaDocumentKind::SqlDdl,
            document: "db/schema.sql".to_string(),
            objects: vec![SchemaObject {
                identity: ObjectId::new("public"),
                kind: SchemaObjectKind::Namespace,
                semantic: BTreeMap::new(),
                dependencies: BTreeSet::new(),
                source: Some(SchemaSourceLocation {
                    document: "db/schema.sql".to_string(),
                    start: 0,
                    end: 21,
                }),
            }],
        }],
    };
    let mut plan = EmbeddedPlan {
        provider_identity: "sifr.sql.postgresql.schema".to_string(),
        protocol_major: 1,
        plan_kind: PlanKind::Document,
        schema_identity: None,
        result_type: ClosedType::None,
        operations: vec![SemanticOperation::ProviderNode {
            tag: SCHEMA_NORMALIZATION_PAYLOAD_TAG.to_string(),
            payload: serde_json::to_vec(&output).expect("schema output"),
        }],
        runtime: RuntimeLowering::NoRuntime,
        dependencies: Vec::new(),
        diagnostics: Vec::new(),
        source_map: Vec::new(),
        stable_fingerprint: String::new(),
    };
    plan.stable_fingerprint = compute_plan_fingerprint(&plan).expect("plan fingerprint");
    serde_json::to_vec(&EmbeddedAnalysisResponse {
        protocol_major: 1,
        plan,
    })
    .expect("schema response")
}

fn fixture_component(output: &[u8]) -> Vec<u8> {
    let escaped = output
        .iter()
        .map(|byte| format!("\\{:02x}", byte))
        .collect::<String>();
    let source = format!(
        r#"(component
            (type $analyze-type (func (param "request" (list u8)) (result (list u8))))
            (core module $module
                (memory (export "memory") 2)
                (data (i32.const 16) "{escaped}")
                (global $next (mut i32) (i32.const 65536))
                (func (export "cabi_realloc")
                    (param $old i32) (param $old-size i32) (param $align i32) (param $new-size i32)
                    (result i32)
                    (local $result i32)
                    global.get $next
                    local.tee $result
                    local.get $new-size
                    i32.add
                    global.set $next
                    local.get $result)
                (func (export "analyze")
                    (param $request i32) (param $request-len i32) (result i32)
                    i32.const 0 i32.const 16 i32.store
                    i32.const 0 i32.const {} i32.store offset=4
                    i32.const 0))
            (core instance $instance (instantiate $module))
            (alias core export $instance "memory" (core memory $memory))
            (alias core export $instance "cabi_realloc" (core func $realloc))
            (alias core export $instance "analyze" (core func $analyze))
            (func $lifted (type $analyze-type)
                (canon lift (core func $analyze) (memory $memory) (realloc $realloc)))
            (export "analyze" (func $lifted)))"#,
        output.len()
    );
    wat::parse_str(source).expect("fixture component")
}

fn cargo_id(name: &str) -> CargoPackageId {
    CargoPackageId(format!("registry+https://example.invalid#{name}@1.0.0"))
}

fn temp_root(label: &str) -> PathBuf {
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("sifr-{label}-{}-{sequence}", std::process::id()))
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

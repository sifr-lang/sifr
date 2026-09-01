use super::sql_application_queries::compile_application_queries;
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
    Cardinality, CodecIdentity, DatabaseType, DialectIdentity, EffectContract, IntegerSign,
    IntegerWidth, Nullability, ObjectId, PROVIDER_ANALYSIS_PAYLOAD_TAG, ProviderAnalysis,
    ProviderParameter, ProviderResultField, QueryEffect, SCHEMA_NORMALIZATION_PAYLOAD_TAG,
    SchemaDocument, SchemaDocumentKind, SchemaNormalizationOutput, SchemaObject, SchemaObjectKind,
    SchemaSourceLocation, SifrType,
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
    let first = prepared
        .cache_fragment()
        .expect("prepared profile cache invariant");
    std::fs::write(
        fixture.owner_root.join("db/schema.sql"),
        "create schema public;\n-- semantic source change\n",
    )
    .expect("update checked-in source");
    let second = prepare_sql_profiles(&fixture.graph, &fixture.owner_id)
        .expect("changed checked-in source should still normalize")
        .cache_fragment()
        .expect("changed profile cache invariant");
    assert_ne!(first, second);
    assert!(first.contains("sifr.sql.schemas.app"));
    assert_eq!(prepared.requirements().len(), 1);
    let requirement_name = format!("{}::has_users", fixture.owner_id.0);
    let requirement = prepared
        .requirements()
        .requirement(&requirement_name)
        .expect("prepared requirement");
    let profile = prepared
        .registry()
        .profile("app")
        .expect("prepared profile");
    let proof = requirement
        .prove(profile.authority())
        .expect("profile must prove requirement");
    assert_eq!(proof.provider_family, "postgresql");
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

#[test]
fn package_query_declarations_emit_non_empty_compatibility_artifact() {
    let fixture = profile_fixture_with_components(
        fixture_component(&schema_response()),
        fixture_component(&query_response()),
    );
    let prepared = prepare_sql_profiles(&fixture.graph, &fixture.owner_id)
        .expect("offline schema profile preparation should succeed");
    let source = "from sifr.sql.schemas import app\n\n@app.query\ndef find_user(user_id: int64) -> Template:\n    return app.sql(t\"SELECT {user_id} AS value\")\n";
    let parsed = crate::frontend::parse_source(source).expect("query source should parse");
    let mut external_defs = crate::stdlib::external_defs().expect("stdlib should compile");
    prepared.install_compiler_externals(&mut external_defs);
    let mut project = crate::project::compile_single_frontend_module_with_source_and_options(
        "main",
        &parsed,
        sifr_frontend::FrontendSourceContext {
            display_path: "src/main.sifr",
            source,
        },
        external_defs,
        sifr_frontend::FrontendDiagnosticStyle::Bare,
        sifr_lowering::LoweringOptions::default(),
    )
    .expect("profile query source should lower through the normal frontend");
    let registry = compile_application_queries(&mut project, &prepared)
        .expect("profile query should compile through the resolved component");
    let artifact = registry
        .exported_artifact("fixture-package")
        .expect("query signatures should export");
    assert_eq!(artifact.entries.len(), 1);
    assert_eq!(
        artifact
            .entries
            .values()
            .next()
            .map(|entry| entry.symbol.as_str()),
        Some("find_user")
    );
    let module = project
        .hir_modules
        .get("main")
        .expect("lowered main module");
    assert!(
        module
            .imports
            .iter()
            .all(|import| import.module != "sifr.sql.schemas")
    );
}

#[test]
fn portable_schema_source_lowers_and_runtime_witness_uses_fail() {
    let fixture = profile_fixture_with_components(
        fixture_component(&schema_response()),
        fixture_component(&portable_query_response()),
    );
    let prepared = prepare_sql_profiles(&fixture.graph, &fixture.owner_id)
        .expect("offline schema profile preparation should succeed");
    let mut external_defs = crate::stdlib::external_defs().expect("stdlib should compile");
    prepared.install_compiler_externals(&mut external_defs);
    let positive = include_str!(
        "../../../../verification/areas/sql_platform/fixtures/schema_polymorphism/portable_by_email.sifr"
    );
    let mut project = lower_sql_fixture(positive, external_defs.clone())
        .expect("portable SqlSchema source should lower through the normal frontend");
    let registry = compile_application_queries(&mut project, &prepared)
        .expect("portable source query must specialize through its concrete profile");
    let module = project
        .hir_modules
        .get("main")
        .expect("portable main module");
    assert!(
        module
            .functions
            .iter()
            .all(|function| function.name != "by_email")
    );
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__sifr_sql_by_email_"))
        .expect("profile-specialized function");
    assert_eq!(specialized.params.len(), 1);
    assert_eq!(specialized.params[0].name, "email");
    assert!(specialized.type_params.is_empty());
    registry
        .row_of("main", &specialized.name, false)
        .expect("specialized query owns a proof-backed contract");
    assert!(module.imports.iter().all(|import| {
        import.module != "sifr.sql"
            && import.module != "sifr.sql.schemas"
            && import.module != "sifr.sql.requirements"
    }));

    for (case, negative, expected_code) in [
        (
            "return",
            include_str!(
                "../../../../verification/areas/sql_platform/fixtures/schema_polymorphism/negative_return_witness.sifr"
            ),
            sifr_diagnostics::DiagnosticCode::SQL_PROVIDER_CONTRACT.code(),
        ),
        (
            "store",
            include_str!(
                "../../../../verification/areas/sql_platform/fixtures/schema_polymorphism/negative_store_witness.sifr"
            ),
            sifr_diagnostics::DiagnosticCode::SQL_PROVIDER_CONTRACT.code(),
        ),
        (
            "capture",
            include_str!(
                "../../../../verification/areas/sql_platform/fixtures/schema_polymorphism/negative_capture_witness.sifr"
            ),
            sifr_diagnostics::DiagnosticCode::SQL_PROVIDER_CONTRACT.code(),
        ),
        (
            "unconstrained",
            include_str!(
                "../../../../verification/areas/sql_platform/fixtures/schema_polymorphism/negative_unconstrained_witness.sifr"
            ),
            sifr_diagnostics::DiagnosticCode::SQL_PROVIDER_CONTRACT.code(),
        ),
        (
            "concrete",
            include_str!(
                "../../../../verification/areas/sql_platform/fixtures/schema_polymorphism/negative_concrete_parameter_witness.sifr"
            ),
            sifr_diagnostics::DiagnosticCode::SQL_PROVIDER_CONTRACT.code(),
        ),
    ] {
        let diagnostics = match lower_sql_fixture(negative, external_defs.clone()) {
            Ok(_) => panic!("runtime SqlSchema witness use must fail"),
            Err(diagnostics) => diagnostics,
        };
        assert_eq!(
            diagnostics[0].code, expected_code,
            "negative portable witness case: {case}"
        );
    }

    let invalid_direct = include_str!(
        "../../../../verification/areas/sql_platform/fixtures/schema_polymorphism/negative_standalone_profile_witness.sifr"
    );
    let mut project = lower_sql_fixture(invalid_direct, external_defs)
        .expect("direct profile witness misuse reaches application query compilation");
    let diagnostics = compile_application_queries(&mut project, &prepared)
        .expect_err("standalone profile witness must not reach runtime HIR");
    assert_eq!(
        diagnostics[0].code,
        sifr_diagnostics::DiagnosticCode::COMPONENT_EXECUTION.code()
    );
}

#[test]
fn migration_sifr_source_discovers_checked_ordered_steps() {
    let source = include_str!(
        "../../../../verification/areas/sql_platform/fixtures/migration_source/add_status.sifr"
    );
    let external_defs = crate::stdlib::external_defs().expect("stdlib should compile");
    let project = lower_sql_fixture(source, external_defs)
        .expect("migration source should lower through the normal frontend");
    let declarations = sifr_frontend::sql_migration_declarations(
        project.hir_modules.get("main").expect("migration module"),
    )
    .expect("migration source declaration should compile");
    assert_eq!(declarations.len(), 1);
    assert_eq!(declarations[0].id.as_str(), "2026_08_add_status");
    assert_eq!(declarations[0].steps.len(), 4);
    assert!(
        declarations[0]
            .rollback
            .as_ref()
            .is_some_and(|steps| steps.len() == 1)
    );
    assert!(matches!(
        &declarations[0].steps[1].kind,
        sifr_frontend::MigrationSourceStepKind::SqlData { statement }
            if statement.contains("'pending'")
    ));
}

fn lower_sql_fixture(
    source: &str,
    external_defs: sifr_lowering::ExternalDefs,
) -> Result<crate::project::ProjectLowering, Vec<crate::diagnostics::RenderedDiagnostic>> {
    let parsed = crate::frontend::parse_source(source)?;
    crate::project::compile_single_frontend_module_with_source_and_options(
        "main",
        &parsed,
        sifr_frontend::FrontendSourceContext {
            display_path: "fixture.sifr",
            source,
        },
        external_defs,
        sifr_frontend::FrontendDiagnosticStyle::Bare,
        sifr_lowering::LoweringOptions::default(),
    )
}

struct ProfileFixture {
    graph: SifrPackageGraph,
    owner_id: SifrPackageId,
    owner_root: PathBuf,
}

fn profile_fixture(component_bytes: Vec<u8>) -> ProfileFixture {
    profile_fixture_with_components(component_bytes.clone(), component_bytes)
}

fn profile_fixture_with_components(
    schema_component_bytes: Vec<u8>,
    query_component_bytes: Vec<u8>,
) -> ProfileFixture {
    let root = temp_root("sql-profile");
    let owner_root = root.join("app");
    let provider_root = root.join("postgres");
    std::fs::create_dir_all(owner_root.join("db")).expect("owner schema directory");
    std::fs::create_dir_all(provider_root.join("components"))
        .expect("provider component directory");
    std::fs::write(owner_root.join("db/schema.sql"), "create schema public;\n")
        .expect("schema source");
    std::fs::write(
        provider_root.join("components/postgresql-schema.wasm"),
        &schema_component_bytes,
    )
    .expect("component artifact");
    std::fs::write(
        provider_root.join("components/postgresql-query.wasm"),
        &query_component_bytes,
    )
    .expect("query component artifact");
    let schema_component_sha = lower_hex(&Sha256::digest(&schema_component_bytes));
    let query_component_sha = lower_hex(&Sha256::digest(&query_component_bytes));
    let owner = package("app", &owner_root, parse_manifest(&owner_manifest()));
    let provider = package(
        "postgres",
        &provider_root,
        parse_manifest(&provider_manifest(
            &schema_component_sha,
            &query_component_sha,
        )),
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
family = "postgresql"
source = "db/schema.sql"
server-version = "18"
search-path = ["public"]
extensions = ["citext"]
sql-modes = ["standard"]
pooling = "session"
schema-evidence = "migration-head"
schema-strictness = "compatible"

[sql.requirements.has_users]
capabilities = ["sql.bind.parameters", "sql.expression.equality", "sql.query.select"]

[sql.requirements.has_users.providers.postgresql]
provider = "postgres"
source = "db/schema.sql"
server-version = "18"
extensions = ["citext"]
sql-modes = ["standard"]
"#
    .to_string()
}

fn provider_manifest(schema_sha256: &str, query_sha256: &str) -> String {
    format!(
        r#"[package]
name = "sifr_sql_postgresql"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[compiler-components.postgresql-schema]
kind = "embedded-language-provider"
artifact = "components/postgresql-schema.wasm"
version = "1.0.0"
sha256 = "{schema_sha256}"
protocol-min = 1
protocol-max = 1
processors = ["sifr.sql.postgresql.schema"]
diagnostic-namespace = "SQL-POSTGRESQL"
diagnostics = [{{ code = "SIFR-SQL-POSTGRESQL-0001", lifecycle = "active" }}]

[compiler-components.postgresql-query]
kind = "embedded-language-provider"
artifact = "components/postgresql-query.wasm"
version = "1.0.0"
sha256 = "{query_sha256}"
protocol-min = 1
protocol-max = 1
processors = ["sifr.sql.postgresql.sql"]
diagnostic-namespace = "SQL-POSTGRESQL-QUERY"
diagnostics = [{{ code = "SIFR-SQL-POSTGRESQL-QUERY-0001", lifecycle = "active" }}]
"#
    )
}

fn query_response() -> Vec<u8> {
    let codec = CodecIdentity::new("postgresql.int64.binary.v1").expect("codec identity");
    let database_type = DatabaseType::Integer {
        sign: IntegerSign::Signed,
        width: IntegerWidth::Bits64,
    };
    let sifr_type = SifrType::FixedInteger {
        sign: IntegerSign::Signed,
        width: IntegerWidth::Bits64,
    };
    let analysis = ProviderAnalysis {
        server_profile: "postgresql-18".to_string(),
        normalized_statement: "SELECT $1::bigint AS value".to_string(),
        parameters: vec![ProviderParameter {
            slot: 0,
            database_type: database_type.clone(),
            nullability: Nullability::NonNull,
            codec: codec.clone(),
        }],
        result_fields: vec![ProviderResultField {
            name: "value".to_string(),
            sifr_type: sifr_type.clone(),
            database_type,
            nullability: Nullability::NonNull,
            codec,
            source_object: None,
        }],
        cardinality: Cardinality::EXACTLY_ONE,
        effects: EffectContract::new(QueryEffect::Read, BTreeSet::new(), BTreeSet::new())
            .expect("read effect"),
        accessed_objects: BTreeSet::new(),
        semantic_flags: BTreeSet::from(["stable-result-name".to_string()]),
        required_capabilities: BTreeSet::from([
            "sql.bind.parameters".to_string(),
            "sql.query.select".to_string(),
        ]),
    };
    let mut plan = EmbeddedPlan {
        provider_identity: "sifr.sql.postgresql.sql".to_string(),
        protocol_major: 1,
        plan_kind: PlanKind::Expression,
        schema_identity: None,
        result_type: ClosedType::None,
        operations: vec![SemanticOperation::ProviderNode {
            tag: PROVIDER_ANALYSIS_PAYLOAD_TAG.to_string(),
            payload: serde_json::to_vec(&analysis).expect("provider analysis"),
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
    .expect("query response")
}

fn portable_query_response() -> Vec<u8> {
    let text_codec = CodecIdentity::new("postgresql.text.binary.v1").expect("codec identity");
    let int_codec = CodecIdentity::new("postgresql.int64.binary.v1").expect("codec identity");
    let text_type = DatabaseType::Text {
        fixed: false,
        max_characters: None,
    };
    let int_type = DatabaseType::Integer {
        sign: IntegerSign::Signed,
        width: IntegerWidth::Bits64,
    };
    let analysis = ProviderAnalysis {
        server_profile: "postgresql-18".to_string(),
        normalized_statement: "SELECT id, email FROM public.users WHERE email = $1::text"
            .to_string(),
        parameters: vec![ProviderParameter {
            slot: 0,
            database_type: text_type.clone(),
            nullability: Nullability::NonNull,
            codec: text_codec.clone(),
        }],
        result_fields: vec![
            ProviderResultField {
                name: "id".to_string(),
                sifr_type: SifrType::FixedInteger {
                    sign: IntegerSign::Signed,
                    width: IntegerWidth::Bits64,
                },
                database_type: int_type,
                nullability: Nullability::NonNull,
                codec: int_codec,
                source_object: Some(ObjectId::new("public.users.id")),
            },
            ProviderResultField {
                name: "email".to_string(),
                sifr_type: SifrType::Str,
                database_type: text_type,
                nullability: Nullability::NonNull,
                codec: text_codec,
                source_object: Some(ObjectId::new("public.users.email")),
            },
        ],
        cardinality: Cardinality::MANY,
        effects: EffectContract::new(QueryEffect::Read, BTreeSet::new(), BTreeSet::new())
            .expect("read effect"),
        accessed_objects: BTreeSet::from([
            ObjectId::new("public.users"),
            ObjectId::new("public.users.id"),
            ObjectId::new("public.users.email"),
        ]),
        semantic_flags: BTreeSet::from([
            "deterministic-order".to_string(),
            "stable-result-name".to_string(),
        ]),
        required_capabilities: BTreeSet::from([
            "sql.bind.parameters".to_string(),
            "sql.expression.equality".to_string(),
            "sql.query.select".to_string(),
        ]),
    };
    let mut plan = EmbeddedPlan {
        provider_identity: "sifr.sql.postgresql.sql".to_string(),
        protocol_major: 1,
        plan_kind: PlanKind::Expression,
        schema_identity: None,
        result_type: ClosedType::None,
        operations: vec![SemanticOperation::ProviderNode {
            tag: PROVIDER_ANALYSIS_PAYLOAD_TAG.to_string(),
            payload: serde_json::to_vec(&analysis).expect("provider analysis"),
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
    .expect("query response")
}

fn schema_response() -> Vec<u8> {
    let output = SchemaNormalizationOutput {
        dialect: DialectIdentity {
            family: "postgresql".to_string(),
            server_version: "18".to_string(),
            modes: BTreeSet::from(["standard".to_string()]),
            features: BTreeSet::from(["citext".to_string()]),
        },
        capabilities: BTreeSet::from([
            "sql.bind.parameters".to_string(),
            "sql.expression.equality".to_string(),
            "sql.query.select".to_string(),
        ]),
        documents: vec![SchemaDocument {
            kind: SchemaDocumentKind::SqlDdl,
            document: "db/schema.sql".to_string(),
            objects: vec![
                schema_object("public", SchemaObjectKind::Namespace, BTreeSet::new()),
                schema_object(
                    "public.users",
                    SchemaObjectKind::Table,
                    BTreeSet::from([ObjectId::new("public")]),
                ),
                schema_object(
                    "public.users.id",
                    SchemaObjectKind::Column,
                    BTreeSet::from([ObjectId::new("public.users")]),
                ),
                schema_object(
                    "public.users.email",
                    SchemaObjectKind::Column,
                    BTreeSet::from([ObjectId::new("public.users")]),
                ),
            ],
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

fn schema_object(
    identity: &str,
    kind: SchemaObjectKind,
    dependencies: BTreeSet<ObjectId>,
) -> SchemaObject {
    SchemaObject {
        identity: ObjectId::new(identity),
        kind,
        semantic: BTreeMap::new(),
        dependencies,
        source: Some(SchemaSourceLocation {
            document: "db/schema.sql".to_string(),
            start: 0,
            end: 21,
        }),
    }
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

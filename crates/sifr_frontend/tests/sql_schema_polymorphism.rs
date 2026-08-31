#![allow(clippy::expect_used, clippy::unwrap_used)]

use semver::Version;
use sifr_frontend::{
    QueryCompilationInput, SchemaPolymorphicQueryCompiler, SchemaSpecializationInput,
    SqlExecutionResourceKind, SqlQueryCompiler, SqlSchemaWitness, SqlSchemaWitnessUse,
    VerifiedSqlExecutionResource, validate_sql_schema_witness_use,
};
use sifr_ir::{HirExpr, HirSqlExecutionMethod};
use sifr_sql_contract::{
    Cardinality, CodecContract, CodecIdentity, CodecRegistry, DatabaseType, DialectIdentity,
    EffectContract, IntegerSign, IntegerWidth, NullCodecBehavior, Nullability, ObjectId,
    PanicContainment, PoolingMode, ProfileModuleRegistry, ProviderAnalysis, ProviderIdentity,
    ProviderParameter, ProviderResultField, QueryEffect, QueryOrigin, SchemaDocument,
    SchemaDocumentKind, SchemaEvidence, SchemaObject, SchemaObjectKind, SchemaProfile,
    SchemaRequirement, SchemaRequirementErrorKind, SchemaRequirementIdentity,
    SchemaRequirementRegistry, SchemaSourceLocation, SchemaStrictness, SemanticValue,
    SessionContract, SifrType, WireFormatIdentity, build_profile_authority,
    build_provider_schema_requirement, generate_profile_module, normalize_schema,
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn specialization_erases_witness_and_binds_execution_to_the_proving_profile() {
    let (profiles, requirements, codecs) = environment();
    let app = profiles.profile("app").unwrap();
    let witness = SqlSchemaWitness::from_profile_export(app);
    let compiler = SchemaPolymorphicQueryCompiler::new(&profiles, &requirements);
    let specialized = compiler
        .specialize(SchemaSpecializationInput {
            requirement_name: "library::has_users",
            profile_name: "app",
            witness: &witness,
            query: query_input(&codecs, "app", ObjectId::new("public.users")),
        })
        .expect("specialization must prove the concrete profile");
    assert_eq!(
        specialized.query.contract.profile_identity,
        app.authority().nominal_identity
    );
    assert_eq!(
        specialized.proof.profile_identity,
        witness.profile_identity()
    );

    let query_compiler = SqlQueryCompiler::new(&profiles);
    let bound = query_compiler
        .bind(&specialized.query, vec![HirExpr::BoolLiteral(true)])
        .unwrap();
    let app_pool = VerifiedSqlExecutionResource::from_profile(app, SqlExecutionResourceKind::Pool);
    assert!(
        query_compiler
            .execution(bound, &app_pool, HirSqlExecutionMethod::FetchOptional)
            .is_ok()
    );

    let bound = query_compiler
        .bind(&specialized.query, vec![HirExpr::BoolLiteral(true)])
        .unwrap();
    let other_transaction = VerifiedSqlExecutionResource::from_profile(
        profiles.profile("other").unwrap(),
        SqlExecutionResourceKind::Transaction,
    );
    assert!(
        query_compiler
            .execution(
                bound,
                &other_transaction,
                HirSqlExecutionMethod::FetchOptional,
            )
            .is_err()
    );
}

#[test]
fn every_runtime_witness_use_is_rejected() {
    let identity = SchemaRequirementIdentity::new("library", "has_users").unwrap();
    for use_site in [
        SqlSchemaWitnessUse::RuntimeStorage,
        SqlSchemaWitnessUse::Return,
        SqlSchemaWitnessUse::Capture,
        SqlSchemaWitnessUse::Selection,
        SqlSchemaWitnessUse::UnconstrainedGenericParameter,
    ] {
        let error =
            validate_sql_schema_witness_use(&use_site).expect_err("runtime witness use must fail");
        assert_eq!(error.kind, SchemaRequirementErrorKind::InvalidWitnessUse);
    }
    assert!(
        validate_sql_schema_witness_use(&SqlSchemaWitnessUse::DirectNamespaceExport {
            module_path: "sifr.sql.schemas.app".to_string(),
            export_name: "schema".to_string(),
        })
        .is_ok()
    );
    assert!(
        validate_sql_schema_witness_use(&SqlSchemaWitnessUse::ConstrainedGenericParameter {
            requirement: identity,
        })
        .is_ok()
    );
}

#[test]
fn undeclared_objects_and_provider_behavior_fail_before_query_lowering() {
    let (profiles, requirements, codecs) = environment();
    let witness = SqlSchemaWitness::from_profile_export(profiles.profile("app").unwrap());
    let compiler = SchemaPolymorphicQueryCompiler::new(&profiles, &requirements);
    let object_error = compiler
        .specialize(SchemaSpecializationInput {
            requirement_name: "library::has_users",
            profile_name: "app",
            witness: &witness,
            query: query_input(&codecs, "app", ObjectId::new("public.audit")),
        })
        .expect_err("undeclared table must fail");
    assert_eq!(
        object_error.kind,
        SchemaRequirementErrorKind::UndeclaredObject
    );

    let mut behavior_query = query_input(&codecs, "app", ObjectId::new("public.users"));
    behavior_query
        .analysis
        .required_capabilities
        .insert("sql.query.window".to_string());
    let behavior_error = compiler
        .specialize(SchemaSpecializationInput {
            requirement_name: "library::has_users",
            profile_name: "app",
            witness: &witness,
            query: behavior_query,
        })
        .expect_err("undeclared provider behavior must fail");
    assert_eq!(
        behavior_error.kind,
        SchemaRequirementErrorKind::UndeclaredBehavior
    );
}

fn environment() -> (
    ProfileModuleRegistry,
    SchemaRequirementRegistry,
    CodecRegistry,
) {
    let schema = schema();
    let mut profiles = ProfileModuleRegistry::default();
    for name in ["app", "other"] {
        let authority = authority(name, schema.clone());
        profiles
            .register(
                authority.clone(),
                generate_profile_module(&authority).unwrap(),
            )
            .unwrap();
    }
    let identity = SchemaRequirementIdentity::new("library", "has_users").unwrap();
    let artifact = build_provider_schema_requirement(
        identity.clone(),
        "db/requirements/has_users.postgresql.sql",
        "a".repeat(64),
        &schema,
        capabilities(),
        &provider_capabilities(),
    )
    .unwrap();
    let mut requirements = SchemaRequirementRegistry::default();
    requirements
        .register(SchemaRequirement::new(identity, [artifact]).unwrap())
        .unwrap();
    (profiles, requirements, codecs())
}

fn query_input<'a>(
    codecs: &'a CodecRegistry,
    profile_name: &'a str,
    relation: ObjectId,
) -> QueryCompilationInput<'a> {
    QueryCompilationInput {
        profile_name,
        origin: QueryOrigin::new("queries.users", "by_active", 1, 80).unwrap(),
        analysis: ProviderAnalysis {
            server_profile: "postgresql-18".to_string(),
            normalized_statement: "SELECT id FROM public.users WHERE active = $1".to_string(),
            parameters: vec![ProviderParameter {
                slot: 0,
                database_type: DatabaseType::Boolean,
                nullability: Nullability::NonNull,
                codec: CodecIdentity::new("postgresql.bool.v1").unwrap(),
            }],
            result_fields: vec![ProviderResultField {
                name: "id".to_string(),
                sifr_type: SifrType::FixedInteger {
                    sign: IntegerSign::Signed,
                    width: IntegerWidth::Bits64,
                },
                database_type: DatabaseType::Integer {
                    sign: IntegerSign::Signed,
                    width: IntegerWidth::Bits64,
                },
                nullability: Nullability::NonNull,
                codec: CodecIdentity::new("postgresql.int8.v1").unwrap(),
                source_object: Some(ObjectId::new("public.users.id")),
            }],
            cardinality: Cardinality::AT_MOST_ONE,
            effects: EffectContract::new(
                QueryEffect::Read,
                BTreeSet::from([relation]),
                BTreeSet::new(),
            )
            .unwrap(),
            semantic_flags: BTreeSet::new(),
            required_capabilities: capabilities(),
        },
        codecs,
        parameter_types: vec![SifrType::Bool],
        deterministic_order: true,
        fragment_identities: Vec::new(),
    }
}

fn authority(
    name: &str,
    schema: sifr_sql_contract::SchemaIr,
) -> sifr_sql_contract::ProfileAuthority {
    build_profile_authority(SchemaProfile {
        package_id: "app@1.0.0#registry".to_string(),
        name: name.to_string(),
        source_files: BTreeSet::from(["db/schema.sql".to_string()]),
        source_fingerprints: BTreeMap::from([("db/schema.sql".to_string(), "b".repeat(64))]),
        evidence: SchemaEvidence::MigrationHead,
        strictness: SchemaStrictness::Compatible,
        pooling: PoolingMode::Session,
        session: SessionContract::default(),
        accepted_signers: BTreeSet::new(),
        capabilities: provider_capabilities(),
        schema,
    })
    .unwrap()
}

fn schema() -> sifr_sql_contract::SchemaIr {
    let source = "db/requirements/has_users.postgresql.sql";
    normalize_schema(
        provider(),
        DialectIdentity {
            family: "postgresql".to_string(),
            server_version: "18".to_string(),
            modes: BTreeSet::new(),
            features: BTreeSet::new(),
        },
        [SchemaDocument {
            kind: SchemaDocumentKind::SqlDdl,
            document: source.to_string(),
            objects: vec![
                object(
                    "public",
                    SchemaObjectKind::Namespace,
                    BTreeMap::new(),
                    [],
                    source,
                ),
                object(
                    "public.users",
                    SchemaObjectKind::Table,
                    BTreeMap::new(),
                    ["public"],
                    source,
                ),
                object(
                    "public.users.id",
                    SchemaObjectKind::Column,
                    BTreeMap::from([
                        ("type".to_string(), SemanticValue::Text("int8".to_string())),
                        ("nullable".to_string(), SemanticValue::Bool(false)),
                    ]),
                    ["public.users"],
                    source,
                ),
                object(
                    "public.users.active",
                    SchemaObjectKind::Column,
                    BTreeMap::from([
                        ("type".to_string(), SemanticValue::Text("bool".to_string())),
                        ("nullable".to_string(), SemanticValue::Bool(false)),
                    ]),
                    ["public.users"],
                    source,
                ),
            ],
        }],
    )
    .unwrap()
}

fn object<const N: usize>(
    identity: &str,
    kind: SchemaObjectKind,
    semantic: BTreeMap<String, SemanticValue>,
    dependencies: [&str; N],
    source: &str,
) -> SchemaObject {
    SchemaObject {
        identity: ObjectId::new(identity),
        kind,
        semantic,
        dependencies: dependencies.into_iter().map(ObjectId::new).collect(),
        source: Some(SchemaSourceLocation {
            document: source.to_string(),
            start: 1,
            end: 2,
        }),
    }
}

fn capabilities() -> BTreeSet<String> {
    BTreeSet::from([
        "sql.bind.parameters".to_string(),
        "sql.expression.equality".to_string(),
        "sql.query.select".to_string(),
    ])
}

fn provider_capabilities() -> BTreeSet<String> {
    BTreeSet::from([
        "sql.bind.parameters".to_string(),
        "sql.expression.equality".to_string(),
        "sql.query.select".to_string(),
        "sql.query.window".to_string(),
    ])
}

fn provider() -> ProviderIdentity {
    ProviderIdentity {
        package_id: "sifr-sql-postgresql@1.0.0#registry".to_string(),
        package_version: Version::new(1, 0, 0),
        package_source: "registry".to_string(),
        package_graph_digest: "fnv1a64:fixture".to_string(),
        compiler_components: BTreeMap::from([("postgresql@1.0.0".to_string(), "c".repeat(64))]),
    }
}

fn codecs() -> CodecRegistry {
    CodecRegistry::for_profile(
        "postgresql-18",
        [
            codec_contract("postgresql.bool.v1", DatabaseType::Boolean, SifrType::Bool),
            codec_contract(
                "postgresql.int8.v1",
                DatabaseType::Integer {
                    sign: IntegerSign::Signed,
                    width: IntegerWidth::Bits64,
                },
                SifrType::FixedInteger {
                    sign: IntegerSign::Signed,
                    width: IntegerWidth::Bits64,
                },
            ),
        ],
    )
    .unwrap()
}

fn codec_contract(
    identity: &str,
    database_type: DatabaseType,
    sifr_type: SifrType,
) -> CodecContract {
    CodecContract {
        identity: CodecIdentity::new(identity).unwrap(),
        database_type,
        sifr_type,
        server_profiles: BTreeSet::from(["postgresql-18".to_string()]),
        encode_error: "sifr.sql.EncodeError".to_string(),
        decode_error: "sifr.sql.DecodeError".to_string(),
        null_behavior: NullCodecBehavior::PassThrough,
        wire_format: WireFormatIdentity::new(format!("{identity}.wire")).unwrap(),
        panic_containment: PanicContainment::CatchAndRedact,
    }
}

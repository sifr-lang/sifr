#![allow(clippy::expect_used, clippy::unwrap_used)]

use semver::Version;
use sifr_frontend::{
    QueryCompilationInput, SqlExecutionResourceKind, SqlQueryCompiler,
    VerifiedSqlExecutionResource, unify_query_value_types,
};
use sifr_ir::{HirExpr, HirSqlEffectKind, HirSqlExecutionMethod};
use sifr_sql_contract::{
    Cardinality, CodecContract, CodecIdentity, CodecRegistry, DatabaseType, DialectIdentity,
    EffectContract, IntegerSign, IntegerWidth, NullCodecBehavior, Nullability, ObjectId,
    PanicContainment, PoolingMode, ProfileModuleRegistry, ProviderAnalysis, ProviderIdentity,
    ProviderParameter, ProviderResultField, QueryEffect, QueryOrigin, SchemaDocument,
    SchemaDocumentKind, SchemaEvidence, SchemaProfile, SchemaStrictness, SessionContract, SifrType,
    WireFormatIdentity, build_profile_authority, generate_profile_module, normalize_schema,
};
use sifr_sql_runtime::{
    BoundParameters, ExecutionMetadata, ExecutionMode, QueryTemplate as RuntimeQueryTemplate,
    RuntimeCardinality, RuntimeEffect, RuntimeEffectContract,
};
use sifr_type_system::{FixedIntType, Type};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn production_query_compiler_resolves_profiles_and_lowers_normal_sifr_types() {
    let (registry, codecs) = registry_and_codecs();
    let compiler = SqlQueryCompiler::new(&registry);
    assert_eq!(
        compiler.profile("app").unwrap().authority().profile.name,
        "app"
    );

    let compiled = compiler
        .compile(query_input(&codecs, Cardinality::AT_MOST_ONE))
        .expect("provider analysis should lower");
    assert_eq!(compiled.hir.parameters[0].ty, Type::Bool);
    let Type::StructuralRecord(row) = &compiled.hir.row_type else {
        panic!("query result must use a structural record")
    };
    assert_eq!(
        row.field("id").expect("id field").ty(),
        &Type::FixedInt(FixedIntType::I64)
    );
    let Type::Class { identity, .. } = &compiled.hir.ty else {
        panic!("query template must use the normal nominal type machinery")
    };
    assert_eq!(identity.as_deref(), Some("sifr.sql.QueryTemplate"));
    assert_eq!(
        unify_query_value_types(&compiled.hir.ty, &compiled.hir.ty),
        compiled.hir.ty
    );
    let changed = unify_query_value_types(&compiled.hir.ty, &Type::Str);
    assert!(matches!(changed, Type::Union(_)));
}

#[test]
fn binding_keeps_capture_order_and_execution_round_trips_effects_and_cardinality() {
    let (registry, codecs) = registry_and_codecs();
    let compiler = SqlQueryCompiler::new(&registry);
    let compiled = compiler
        .compile(query_input(&codecs, Cardinality::AT_MOST_ONE))
        .unwrap();
    let bound = compiler
        .bind(&compiled, vec![HirExpr::BoolLiteral(true)])
        .expect("exact capture should bind");
    assert_eq!(bound.captures.len(), 1);
    assert_eq!(bound.effects.effect, HirSqlEffectKind::Read);
    let resource = VerifiedSqlExecutionResource::from_profile(
        compiler.profile("app").unwrap(),
        SqlExecutionResourceKind::Connection,
    );
    let execution = compiler
        .execution(bound, &resource, HirSqlExecutionMethod::FetchOptional)
        .expect("cardinality supports optional fetch");
    assert_eq!(execution.runtime_cardinality, execution.query.cardinality);
    assert_eq!(execution.runtime_effects, execution.query.effects);
    assert_eq!(execution.runtime_cardinality.maximum, Some(1));

    let runtime_cardinality = RuntimeCardinality::new(
        execution.runtime_cardinality.minimum,
        execution.runtime_cardinality.maximum,
    )
    .unwrap();
    let runtime_effects = RuntimeEffectContract::new(
        RuntimeEffect::Read,
        execution.runtime_effects.referenced_objects.clone(),
        execution.runtime_effects.affected_objects.clone(),
    )
    .unwrap();
    let request = RuntimeQueryTemplate::new(
        std::sync::Arc::new(()),
        compiled.hir.normalized_statement.clone(),
        runtime_cardinality,
        runtime_effects,
        true,
        ExecutionMetadata {
            normalized_statement_fingerprint: "a".repeat(64),
            parameter_type_fingerprint: "b".repeat(64),
            result_type_fingerprint: "c".repeat(64),
            schema_fingerprint: compiled.hir.schema_fingerprint.clone(),
        },
    )
    .unwrap()
    .bind(BoundParameters::default())
    .into_execution_request(ExecutionMode::FetchOptional)
    .unwrap();
    assert_eq!(
        request.cardinality.minimum,
        execution.runtime_cardinality.minimum
    );
    assert_eq!(
        request.cardinality.maximum,
        execution.runtime_cardinality.maximum
    );
    assert_eq!(request.effects.effect, RuntimeEffect::Read);
    assert_eq!(
        request.effects.referenced_objects.as_ref(),
        execution.runtime_effects.referenced_objects
    );

    let wrong = compiler.bind(&compiled, vec![HirExpr::StringLiteral("true".to_string())]);
    assert!(wrong.is_err());
}

#[test]
fn execution_method_never_uses_cardinality_to_choose_a_container() {
    let (registry, codecs) = registry_and_codecs();
    let compiler = SqlQueryCompiler::new(&registry);
    let compiled = compiler
        .compile(query_input(&codecs, Cardinality::MANY))
        .unwrap();
    let bound = compiler
        .bind(&compiled, vec![HirExpr::BoolLiteral(true)])
        .unwrap();
    let resource = VerifiedSqlExecutionResource::from_profile(
        compiler.profile("app").unwrap(),
        SqlExecutionResourceKind::Pool,
    );
    assert!(
        compiler
            .execution(bound, &resource, HirSqlExecutionMethod::FetchOptional)
            .is_err()
    );
    let bound = compiler
        .bind(&compiled, vec![HirExpr::BoolLiteral(true)])
        .unwrap();
    assert!(
        compiler
            .execution(
                bound,
                &resource,
                HirSqlExecutionMethod::FetchAll { maximum_rows: 100 },
            )
            .is_ok()
    );
}

fn query_input(codecs: &CodecRegistry, cardinality: Cardinality) -> QueryCompilationInput<'_> {
    let bool_codec = CodecIdentity::new("postgresql.bool.v1").unwrap();
    let int_codec = CodecIdentity::new("postgresql.int8.v1").unwrap();
    QueryCompilationInput {
        profile_name: "app",
        origin: QueryOrigin::new("queries.users", "find_user", 4, 80).unwrap(),
        analysis: ProviderAnalysis {
            server_profile: "postgresql-18".to_string(),
            normalized_statement: "SELECT id FROM users WHERE active = $1".to_string(),
            parameters: vec![ProviderParameter {
                slot: 0,
                database_type: DatabaseType::Boolean,
                nullability: Nullability::NonNull,
                codec: bool_codec,
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
                codec: int_codec,
                source_object: Some(ObjectId::new("public.users.id")),
            }],
            cardinality,
            effects: EffectContract::new(
                QueryEffect::Read,
                BTreeSet::from([ObjectId::new("public.users")]),
                BTreeSet::new(),
            )
            .unwrap(),
            semantic_flags: BTreeSet::new(),
            required_capabilities: BTreeSet::from([
                "sql.bind.parameters".to_string(),
                "sql.expression.equality".to_string(),
                "sql.query.select".to_string(),
            ]),
        },
        codecs,
        parameter_types: vec![SifrType::Bool],
        deterministic_order: true,
        fragment_identities: Vec::new(),
    }
}

fn registry_and_codecs() -> (ProfileModuleRegistry, CodecRegistry) {
    let authority = authority();
    let module = generate_profile_module(&authority).unwrap();
    let mut registry = ProfileModuleRegistry::default();
    registry.register(authority, module).unwrap();
    let contracts = [
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
    ];
    (
        registry,
        CodecRegistry::for_profile("postgresql-18", contracts).unwrap(),
    )
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

fn authority() -> sifr_sql_contract::ProfileAuthority {
    let schema = normalize_schema(
        ProviderIdentity {
            package_id: "sifr-sql-postgresql@1.0.0#registry".to_string(),
            package_version: Version::new(1, 0, 0),
            package_source: "registry+https://github.com/rust-lang/crates.io-index".to_string(),
            package_graph_digest: "sha256:locked-graph".to_string(),
            compiler_components: BTreeMap::from([("postgresql@1.0.0".to_string(), "a".repeat(64))]),
        },
        DialectIdentity {
            family: "postgresql".to_string(),
            server_version: "18".to_string(),
            modes: BTreeSet::new(),
            features: BTreeSet::new(),
        },
        [SchemaDocument {
            kind: SchemaDocumentKind::SqlDdl,
            document: "db/schema.sql".to_string(),
            objects: Vec::new(),
        }],
    )
    .unwrap();
    build_profile_authority(SchemaProfile {
        package_id: "app@1.0.0#registry".to_string(),
        name: "app".to_string(),
        source_files: BTreeSet::from(["db/schema.sql".to_string()]),
        source_fingerprints: BTreeMap::from([("db/schema.sql".to_string(), "b".repeat(64))]),
        evidence: SchemaEvidence::MigrationHead,
        strictness: SchemaStrictness::Compatible,
        pooling: PoolingMode::Session,
        session: SessionContract::default(),
        accepted_signers: BTreeSet::new(),
        capabilities: BTreeSet::from(["sql.query.select".to_string()]),
        schema,
    })
    .unwrap()
}

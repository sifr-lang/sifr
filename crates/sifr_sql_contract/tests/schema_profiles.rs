#![allow(clippy::expect_used, clippy::unwrap_used)]

use semver::Version;
use sifr_compiler_component::{
    ClosedType, ComponentIdentity, ComponentRegistration, DiagnosticRegistry,
    DiagnosticRegistryOwner, EmbeddedAnalysisResponse, EmbeddedPlan, PlanKind, ProtocolRange,
    RuntimeLowering, SemanticOperation, compute_plan_fingerprint,
};
use sifr_sql_contract::{
    AbsenceFact, DialectIdentity, ObjectId, ObjectRequirement, OverloadSetKind, PoolingMode,
    ProviderIdentity, SCHEMA_NORMALIZATION_PAYLOAD_TAG, SchemaDependencyRequest, SchemaDocument,
    SchemaDocumentKind, SchemaEvidence, SchemaIr, SchemaNormalizationOutput, SchemaObject,
    SchemaObjectKind, SchemaProfile, SchemaSlice, SchemaSourceInput, SchemaSourceLocation,
    SchemaStrictness, SemanticValue, SessionContract, build_profile_authority,
    dialect_modes_for_session, generate_profile_module, minimum_schema_slice, normalize_schema,
    schema_context_artifact, schema_fingerprint, schema_normalization_from_response,
    schema_normalization_request, schema_source_fingerprint, semantic_diff,
    verify_compatible_slice,
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn session_character_inputs_are_mysql_dialect_modes_only() {
    let session = SessionContract {
        sql_modes: BTreeSet::from(["strict".to_string()]),
        collation: Some("utf8mb4_0900_ai_ci".to_string()),
        character_set: Some("utf8mb4".to_string()),
        ..SessionContract::default()
    };
    assert_eq!(
        dialect_modes_for_session("postgresql", &session),
        BTreeSet::from(["strict".to_string()])
    );
    assert_eq!(
        dialect_modes_for_session("mysql", &session),
        BTreeSet::from([
            "character-set:utf8mb4".to_string(),
            "collation:utf8mb4_0900_ai_ci".to_string(),
            "strict".to_string(),
        ])
    );
}

#[test]
fn all_provider_source_forms_normalize_into_one_canonical_graph() {
    let schema = normalize_schema(
        provider(),
        dialect(),
        [
            document(
                SchemaDocumentKind::SqlDdl,
                "db/schema.sql",
                vec![namespace()],
            ),
            document(
                SchemaDocumentKind::ProviderMetadata,
                "db/catalog.json",
                vec![table(), column()],
            ),
            document(
                SchemaDocumentKind::GeneratedDefinitions,
                "db/types.json",
                vec![enum_object("public.status")],
            ),
        ],
    )
    .expect("all provider forms should normalize");

    assert_eq!(schema.objects.len(), 4);
    assert_eq!(
        schema.objects[&ObjectId::new("public.users")].kind,
        SchemaObjectKind::Table
    );
    assert_eq!(
        schema.objects[&ObjectId::new("public.status")].kind,
        SchemaObjectKind::Enum
    );
    let inventory = schema
        .objects
        .values()
        .map(|object| format!("{}:{:?}", object.identity, object.kind))
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!(inventory, @r###"
    public:Namespace
    public.status:Enum
    public.users:Table
    public.users.id:Column
    "###);
}

#[test]
fn fingerprint_ignores_document_and_object_order_but_detects_semantic_drift() {
    let first = schema(vec![namespace(), table(), column()]);
    let mut changed_location = column();
    changed_location.source = Some(SchemaSourceLocation {
        document: "schema.sql".to_string(),
        start: 999,
        end: 1_000,
    });
    let reordered = schema(vec![changed_location, table(), namespace()]);
    assert_eq!(
        schema_fingerprint(&first).expect("fingerprint"),
        schema_fingerprint(&reordered).expect("fingerprint")
    );

    let mut changed = reordered;
    changed
        .objects
        .get_mut(&ObjectId::new("public.users.id"))
        .expect("column")
        .semantic
        .insert("nullable".to_string(), SemanticValue::Bool(true));
    assert_ne!(
        schema_fingerprint(&first).expect("fingerprint"),
        schema_fingerprint(&changed).expect("fingerprint")
    );
    let diff = semantic_diff(&first, &changed);
    assert_eq!(diff.objects.len(), 1);
    assert_eq!(diff.objects[0].identity, ObjectId::new("public.users.id"));
}

#[test]
fn every_object_permutation_has_one_schema_fingerprint() {
    let permutations = [
        vec![namespace(), table(), column()],
        vec![namespace(), column(), table()],
        vec![table(), namespace(), column()],
        vec![table(), column(), namespace()],
        vec![column(), namespace(), table()],
        vec![column(), table(), namespace()],
    ];
    let fingerprints = permutations
        .into_iter()
        .map(|objects| schema_fingerprint(&schema(objects)).expect("fingerprint"))
        .collect::<BTreeSet<_>>();
    assert_eq!(fingerprints.len(), 1);
}

#[test]
fn minimum_slice_keeps_requested_properties_transitive_dependencies_and_absence() {
    let schema = schema(vec![namespace(), table(), column()]);
    let slice = minimum_schema_slice(
        &schema,
        [SchemaDependencyRequest {
            identity: ObjectId::new("public.users.id"),
            properties: BTreeSet::from(["storage_type".to_string(), "nullable".to_string()]),
        }],
        [AbsenceFact::MissingObject {
            identity: ObjectId::new("public.users.deleted_at"),
        }],
    )
    .expect("slice");

    assert_eq!(slice.objects.len(), 3);
    assert_eq!(
        slice.objects[&ObjectId::new("public.users.id")]
            .properties
            .len(),
        2
    );
    verify_compatible_slice(&schema, &slice).expect("same schema is compatible");

    let mut drifted = schema;
    drifted.objects.insert(
        ObjectId::new("public.users.deleted_at"),
        object("public.users.deleted_at", SchemaObjectKind::Column),
    );
    assert!(verify_compatible_slice(&drifted, &slice).is_err());
}

#[test]
fn transitive_reachability_expands_a_direct_requirement_and_compatible_edges_may_grow() {
    let schema = schema(vec![namespace(), table(), column()]);
    let slice = minimum_schema_slice(
        &schema,
        [
            SchemaDependencyRequest {
                identity: ObjectId::new("public.users"),
                properties: BTreeSet::new(),
            },
            SchemaDependencyRequest {
                identity: ObjectId::new("public.users.id"),
                properties: BTreeSet::from(["nullable".to_string()]),
            },
        ],
        [],
    )
    .expect("slice");
    assert_eq!(
        slice.objects[&ObjectId::new("public.users")].properties,
        schema.objects[&ObjectId::new("public.users")].semantic
    );
    let mut observed = schema;
    observed
        .objects
        .get_mut(&ObjectId::new("public.users"))
        .expect("table")
        .dependencies
        .insert(ObjectId::new("public.users.id"));
    verify_compatible_slice(&observed, &slice).expect("new dependency edge is compatible");
}

#[test]
fn overload_absence_facts_use_explicit_provider_metadata_for_every_overload_kind() {
    let mut operator = object("public.plus.int4.int4", SchemaObjectKind::Operator);
    operator.semantic.insert(
        "overload_namespace".to_string(),
        SemanticValue::Text("public".to_string()),
    );
    operator.semantic.insert(
        "overload_name".to_string(),
        SemanticValue::Text("plus".to_string()),
    );
    let schema = schema(vec![operator]);
    let candidates = BTreeSet::from([ObjectId::new("public.plus.int4.int4")]);
    let slice = minimum_schema_slice(
        &schema,
        [],
        [AbsenceFact::ExactOverloadSet {
            object_kind: OverloadSetKind::Operator,
            namespace: "public".to_string(),
            name: "plus".to_string(),
            candidates,
        }],
    )
    .expect("overload slice");
    verify_compatible_slice(&schema, &slice).expect("exact operator set");
}

#[test]
fn nominal_profiles_and_generated_namespaces_do_not_collapse_equal_schemas() {
    let first = authority(
        "app",
        schema(vec![
            namespace(),
            enum_object("public.sql"),
            enum_object("app.sql"),
            enum_object("app.class"),
        ]),
    );
    let second = authority("analytics", first.profile.schema.clone());
    assert_ne!(first.nominal_identity, second.nominal_identity);
    assert_ne!(first.profile_fingerprint, second.profile_fingerprint);

    let module = generate_profile_module(&first).expect("generated module");
    assert!(module.source.contains("class Schema:"));
    assert!(module.source.contains("class enums__public__sql(Enum):"));
    assert!(module.source.contains("class enums__app__sql(Enum):"));
    assert!(
        module
            .source
            .contains("class enums__app___sifr_sql_636c617373(Enum):")
    );
    assert!(module.metadata.compiler_known_exports.contains("sql"));
    assert_eq!(
        module
            .lookup_static_symbol(&first, "public.sql")
            .expect("qualified collision lookup"),
        ObjectId::new("public.sql")
    );
    assert!(!module.source.contains("database_url"));
}

#[test]
fn schema_component_round_trip_binds_source_bytes_and_source_kinds() {
    let sources = vec![SchemaSourceInput {
        document: "db/schema.sql".to_string(),
        kind: SchemaDocumentKind::SqlDdl,
        fingerprint: schema_source_fingerprint(b"create schema public;"),
        contents: b"create schema public;".to_vec(),
    }];
    let registration = component_registration();
    let request = schema_normalization_request(
        &registration,
        "0.0.0",
        "app::main",
        "18",
        &SessionContract::default(),
        &BTreeSet::from(["citext".to_string()]),
        &sources,
    )
    .expect("schema request");
    assert_eq!(
        request.context.artifacts[0].fingerprint,
        sources[0].fingerprint
    );
    let output = SchemaNormalizationOutput {
        dialect: dialect(),
        capabilities: BTreeSet::from(["sql.query.select".to_string()]),
        documents: vec![document(
            SchemaDocumentKind::SqlDdl,
            "db/schema.sql",
            vec![namespace()],
        )],
    };
    let mut plan = EmbeddedPlan {
        provider_identity: registration.identity.processor.clone(),
        protocol_major: 1,
        plan_kind: PlanKind::Document,
        schema_identity: None,
        result_type: ClosedType::None,
        operations: vec![SemanticOperation::ProviderNode {
            tag: SCHEMA_NORMALIZATION_PAYLOAD_TAG.to_string(),
            payload: serde_json::to_vec(&output).expect("output"),
        }],
        runtime: RuntimeLowering::NoRuntime,
        dependencies: Vec::new(),
        diagnostics: Vec::new(),
        source_map: Vec::new(),
        stable_fingerprint: String::new(),
    };
    plan.stable_fingerprint = compute_plan_fingerprint(&plan).expect("plan fingerprint");
    let response = EmbeddedAnalysisResponse {
        protocol_major: 1,
        plan,
    };
    let normalized = schema_normalization_from_response(provider(), &sources, &response)
        .expect("normalized response");
    assert!(
        normalized
            .schema
            .objects
            .contains_key(&ObjectId::new("public"))
    );

    let mut wrong_kind = response;
    let SemanticOperation::ProviderNode { payload, .. } = &mut wrong_kind.plan.operations[0] else {
        unreachable!();
    };
    let mut output: SchemaNormalizationOutput =
        serde_json::from_slice(payload).expect("decode output");
    output.documents[0].kind = SchemaDocumentKind::ProviderMetadata;
    *payload = serde_json::to_vec(&output).expect("encode output");
    assert!(schema_normalization_from_response(provider(), &sources, &wrong_kind).is_err());
}

#[test]
fn runtime_manifest_contains_evidence_and_only_the_minimum_slice() {
    let authority = authority("app", schema(vec![namespace(), table(), column()]));
    let slice = SchemaSlice {
        objects: BTreeMap::from([(
            ObjectId::new("public.users"),
            ObjectRequirement {
                identity: ObjectId::new("public.users"),
                kind: SchemaObjectKind::Table,
                properties: BTreeMap::new(),
                dependencies: BTreeSet::from([ObjectId::new("public")]),
            },
        )]),
        absence_facts: BTreeSet::new(),
    };
    let manifest = authority.runtime_manifest(slice.clone());
    assert_eq!(manifest.evidence, SchemaEvidence::MigrationHead);
    assert_eq!(manifest.strictness, SchemaStrictness::Compatible);
    assert_eq!(manifest.dependency_slice, slice);
    let artifact = schema_context_artifact(&authority).expect("schema context artifact");
    assert_eq!(artifact.kind, "sifr.sql.schema-ir");
    assert_eq!(artifact.fingerprint, authority.schema_fingerprint.as_str());
}

fn authority(name: &str, schema: SchemaIr) -> sifr_sql_contract::ProfileAuthority {
    build_profile_authority(SchemaProfile {
        package_id: "app@1.0.0#registry".to_string(),
        name: name.to_string(),
        source_files: BTreeSet::from(["db/schema.sql".to_string()]),
        source_fingerprints: BTreeMap::from([("db/schema.sql".to_string(), "b".repeat(64))]),
        evidence: SchemaEvidence::MigrationHead,
        strictness: SchemaStrictness::Compatible,
        pooling: PoolingMode::Session,
        session: SessionContract {
            search_path: vec!["public".to_string()],
            ..SessionContract::default()
        },
        accepted_signers: BTreeSet::new(),
        capabilities: BTreeSet::from(["sql.query.select".to_string()]),
        schema,
    })
    .expect("profile authority")
}

fn schema(objects: Vec<SchemaObject>) -> SchemaIr {
    normalize_schema(
        provider(),
        dialect(),
        [document(SchemaDocumentKind::SqlDdl, "schema.sql", objects)],
    )
    .expect("schema")
}

fn document(
    kind: SchemaDocumentKind,
    name: &str,
    mut objects: Vec<SchemaObject>,
) -> SchemaDocument {
    for object in &mut objects {
        if let Some(source) = &mut object.source {
            source.document = name.to_string();
        }
    }
    SchemaDocument {
        kind,
        document: name.to_string(),
        objects,
    }
}

fn provider() -> ProviderIdentity {
    ProviderIdentity {
        package_id: "sifr-sql-postgresql@1.0.0#registry".to_string(),
        package_version: Version::new(1, 0, 0),
        package_source: "registry+https://github.com/rust-lang/crates.io-index".to_string(),
        package_graph_digest: "sha256:locked-graph".to_string(),
        compiler_components: BTreeMap::from([("postgresql@1.0.0".to_string(), "a".repeat(64))]),
    }
}

fn dialect() -> DialectIdentity {
    DialectIdentity {
        family: "postgresql".to_string(),
        server_version: "18".to_string(),
        modes: BTreeSet::new(),
        features: BTreeSet::from(["citext".to_string()]),
    }
}

fn component_registration() -> ComponentRegistration {
    ComponentRegistration {
        identity: ComponentIdentity {
            package: "sifr-sql-postgresql@1.0.0#registry".to_string(),
            processor: "sifr.sql.postgresql.schema".to_string(),
            version: Version::new(1, 0, 0),
            sha256: "a".repeat(64),
        },
        protocol: ProtocolRange {
            minimum: 1,
            maximum: 1,
        },
        artifact: "components/postgresql.wasm".to_string(),
        diagnostics: DiagnosticRegistry {
            owner: DiagnosticRegistryOwner::Provider {
                namespace: "SQL-POSTGRESQL".to_string(),
            },
            declarations: Vec::new(),
        },
    }
}

fn namespace() -> SchemaObject {
    object("public", SchemaObjectKind::Namespace)
}

fn table() -> SchemaObject {
    let mut value = object("public.users", SchemaObjectKind::Table);
    value.dependencies.insert(ObjectId::new("public"));
    value
}

fn column() -> SchemaObject {
    let mut value = object("public.users.id", SchemaObjectKind::Column);
    value.dependencies.insert(ObjectId::new("public.users"));
    value.semantic.insert(
        "storage_type".to_string(),
        SemanticValue::Text("int4".to_string()),
    );
    value
        .semantic
        .insert("nullable".to_string(), SemanticValue::Bool(false));
    value
}

fn enum_object(identity: &str) -> SchemaObject {
    let mut value = object(identity, SchemaObjectKind::Enum);
    value.semantic.insert(
        "variants".to_string(),
        SemanticValue::List(vec![
            SemanticValue::Text("active".to_string()),
            SemanticValue::Text("disabled".to_string()),
        ]),
    );
    value
}

fn object(identity: &str, kind: SchemaObjectKind) -> SchemaObject {
    SchemaObject {
        identity: ObjectId::new(identity),
        kind,
        semantic: BTreeMap::new(),
        dependencies: BTreeSet::new(),
        source: Some(SchemaSourceLocation {
            document: "schema.sql".to_string(),
            start: 0,
            end: 1,
        }),
    }
}

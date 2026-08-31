#![allow(clippy::expect_used, clippy::unwrap_used)]

use semver::Version;
use sifr_sql_contract::{
    DialectIdentity, ObjectId, PoolingMode, ProviderIdentity, SchemaDocument, SchemaDocumentKind,
    SchemaEvidence, SchemaObject, SchemaObjectKind, SchemaProfile, SchemaRequirement,
    SchemaRequirementErrorKind, SchemaRequirementIdentity, SchemaRequirementRegistry,
    SchemaSourceLocation, SchemaStrictness, SemanticValue, SessionContract,
    build_profile_authority, build_provider_schema_requirement, normalize_schema,
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn normalized_ddl_becomes_a_complete_structural_requirement() {
    let required = schema("13", false);
    let capabilities = capabilities();
    let identity = SchemaRequirementIdentity::new("library@1.0.0#registry", "has_users").unwrap();
    let artifact = build_provider_schema_requirement(
        identity.clone(),
        "db/requirements/has_users.postgresql.sql",
        "a".repeat(64),
        &required,
        capabilities.clone(),
        &capabilities,
    )
    .expect("normalized DDL requirement");
    assert_eq!(
        artifact.declared_objects(),
        BTreeSet::from([
            ObjectId::new("public"),
            ObjectId::new("public.users"),
            ObjectId::new("public.users.email"),
            ObjectId::new("public.users.id"),
            ObjectId::new("public.users.pk"),
        ])
    );
    let email = &artifact.schema.objects[&ObjectId::new("public.users.email")];
    assert_eq!(
        email.properties["type"],
        SemanticValue::Text("text".to_string())
    );
    assert_eq!(email.properties["nullable"], SemanticValue::Bool(false));
    assert!(artifact.schema.absence_facts.is_empty());

    let requirement = SchemaRequirement::new(identity.clone(), [artifact.clone()]).unwrap();
    let profile = authority(schema("18", true), capabilities.clone());
    let proof = requirement
        .prove(&profile)
        .expect("structural subset proof");
    assert_eq!(proof.requirement, identity);
    assert_eq!(proof.profile_identity, profile.nominal_identity);
    assert_eq!(proof.required_capabilities, capabilities);

    let mut registry = SchemaRequirementRegistry::default();
    registry.register(requirement).unwrap();
    assert!(
        registry
            .requirement("library@1.0.0#registry::has_users")
            .is_ok()
    );
    assert!(
        registry
            .register(SchemaRequirement::new(artifact.identity.clone(), [artifact],).unwrap())
            .is_err()
    );
}

#[test]
fn proof_fails_for_missing_objects_properties_capabilities_and_provider() {
    let required = schema("13", false);
    let identity = SchemaRequirementIdentity::new("library", "has_users").unwrap();
    let artifact = build_provider_schema_requirement(
        identity.clone(),
        "db/requirements/has_users.postgresql.sql",
        "a".repeat(64),
        &required,
        capabilities(),
        &capabilities(),
    )
    .unwrap();
    let requirement = SchemaRequirement::new(identity, [artifact]).unwrap();

    let mut missing = schema("18", true);
    missing.objects.remove(&ObjectId::new("public.users.email"));
    let error = requirement
        .prove(&authority(missing, capabilities()))
        .expect_err("missing column must fail");
    assert_eq!(error.kind, SchemaRequirementErrorKind::IncompatibleSchema);

    let mut changed = schema("18", true);
    changed
        .objects
        .get_mut(&ObjectId::new("public.users.email"))
        .unwrap()
        .semantic
        .insert("nullable".to_string(), SemanticValue::Bool(true));
    assert!(
        requirement
            .prove(&authority(changed, capabilities()))
            .is_err()
    );

    let error = requirement
        .prove(&authority(
            schema("18", true),
            BTreeSet::from(["sql.query.select".to_string()]),
        ))
        .expect_err("missing bind capability must fail");
    assert_eq!(error.kind, SchemaRequirementErrorKind::MissingCapability);

    let mut wrong_provider_schema = schema("18", true);
    wrong_provider_schema.provider.package_id = "another-provider".to_string();
    let error = requirement
        .prove(&authority(wrong_provider_schema, capabilities()))
        .expect_err("different provider identity must fail");
    assert_eq!(error.kind, SchemaRequirementErrorKind::ProviderMismatch);
}

#[test]
fn requirement_fingerprints_are_order_independent_and_reject_tampering() {
    let identity = SchemaRequirementIdentity::new("library", "has_users").unwrap();
    let first = build_provider_schema_requirement(
        identity.clone(),
        "db/requirements/has_users.postgresql.sql",
        "a".repeat(64),
        &schema("13", false),
        capabilities(),
        &capabilities(),
    )
    .unwrap();
    let mut reversed = schema("13", false);
    reversed.objects = reversed.objects.into_iter().rev().collect();
    let second = build_provider_schema_requirement(
        identity,
        "db/requirements/has_users.postgresql.sql",
        "a".repeat(64),
        &reversed,
        capabilities().into_iter().rev().collect(),
        &capabilities(),
    )
    .unwrap();
    assert_eq!(first.artifact_fingerprint, second.artifact_fingerprint);
    let mut tampered = first;
    tampered
        .required_capabilities
        .insert("sql.query.window".to_string());
    assert!(tampered.validate().is_err());
}

fn authority(
    schema: sifr_sql_contract::SchemaIr,
    capabilities: BTreeSet<String>,
) -> sifr_sql_contract::ProfileAuthority {
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
        capabilities,
        schema,
    })
    .unwrap()
}

fn capabilities() -> BTreeSet<String> {
    BTreeSet::from([
        "sql.bind.parameters".to_string(),
        "sql.query.select".to_string(),
    ])
}

fn schema(server_version: &str, add_unrelated: bool) -> sifr_sql_contract::SchemaIr {
    let source = "db/requirements/has_users.postgresql.sql";
    let mut objects = vec![
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
            BTreeMap::from([("name".to_string(), SemanticValue::Text("users".to_string()))]),
            ["public"],
            source,
        ),
        column("public.users.id", "int8", false, source),
        column("public.users.email", "text", false, source),
        object(
            "public.users.pk",
            SchemaObjectKind::PrimaryKey,
            BTreeMap::from([(
                "columns".to_string(),
                SemanticValue::List(vec![SemanticValue::Text("id".to_string())]),
            )]),
            ["public.users", "public.users.id"],
            source,
        ),
    ];
    if add_unrelated {
        objects.push(object(
            "public.audit",
            SchemaObjectKind::Table,
            BTreeMap::new(),
            ["public"],
            source,
        ));
    }
    normalize_schema(
        provider(),
        DialectIdentity {
            family: "postgresql".to_string(),
            server_version: server_version.to_string(),
            modes: BTreeSet::new(),
            features: BTreeSet::new(),
        },
        [SchemaDocument {
            kind: SchemaDocumentKind::SqlDdl,
            document: source.to_string(),
            objects,
        }],
    )
    .unwrap()
}

fn column(identity: &str, ty: &str, nullable: bool, source: &str) -> SchemaObject {
    object(
        identity,
        SchemaObjectKind::Column,
        BTreeMap::from([
            ("type".to_string(), SemanticValue::Text(ty.to_string())),
            ("nullable".to_string(), SemanticValue::Bool(nullable)),
        ]),
        ["public.users"],
        source,
    )
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

fn provider() -> ProviderIdentity {
    ProviderIdentity {
        package_id: "sifr-sql-postgresql@1.0.0#registry".to_string(),
        package_version: Version::new(1, 0, 0),
        package_source: "registry".to_string(),
        package_graph_digest: "fnv1a64:fixture".to_string(),
        compiler_components: BTreeMap::from([("postgresql@1.0.0".to_string(), "c".repeat(64))]),
    }
}

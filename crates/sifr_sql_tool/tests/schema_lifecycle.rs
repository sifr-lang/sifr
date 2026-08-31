#![allow(clippy::expect_used)]

use semver::Version;
use sifr_sql_contract::{
    Cardinality, DialectIdentity, EffectContract, ObjectId, PoolingMode, ProviderIdentity,
    QueryEffect, QuerySignatureArtifact, QuerySignatureEntry, SCHEMA_IR_FORMAT_VERSION,
    SchemaEvidence, SchemaIr, SchemaObject, SchemaObjectKind, SchemaProfile, SchemaStrictness,
    SemanticValue, SessionContract, SifrType, build_profile_authority,
};
use sifr_sql_tool::{
    ARTIFACT_MANIFEST_PATH, AuthorityMergeRule, DEPENDENCY_INDEX_PATH, NamedProfileAuthority,
    NamedSchema, SNAPSHOT_PATH, SchemaLifecycleErrorKind, affected_queries, build_schema_artifacts,
    plan_pull, resolve_build_authority, validate_schema_authorities, write_artifacts_atomically,
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn build_is_deterministic_complete_and_excludes_query_signatures() {
    let authority = authority(schema(false));
    let first = build_schema_artifacts(&authority).expect("first build");
    let second = build_schema_artifacts(&authority).expect("second build");
    assert_eq!(first, second);
    assert!(first.files().contains_key(SNAPSHOT_PATH));
    assert!(first.files().contains_key(DEPENDENCY_INDEX_PATH));
    assert!(first.files().contains_key(ARTIFACT_MANIFEST_PATH));
    assert!(
        first
            .files()
            .keys()
            .all(|path| !path.contains("query-signature"))
    );
    assert_eq!(first.manifest.artifacts.len() + 1, first.files().len());
}

#[test]
fn atomic_write_replaces_one_complete_artifact_set() {
    let temporary = tempfile::tempdir().expect("temporary directory");
    let output = temporary.path().join(".sifr/sql/app");
    let first = build_schema_artifacts(&authority(schema(false))).expect("first build");
    write_artifacts_atomically(&output, &first).expect("first write");
    std::fs::write(output.join("stale"), b"stale").expect("stale file");
    let second = build_schema_artifacts(&authority(schema(true))).expect("second build");
    write_artifacts_atomically(&output, &second).expect("second write");
    assert!(!output.join("stale").exists());
    for (path, expected) in second.files() {
        assert_eq!(
            std::fs::read(output.join(path)).expect("written artifact"),
            *expected
        );
    }
}

#[test]
fn pull_requires_explicit_acceptance_before_replacement() {
    let checked = schema(false);
    let live = schema(true);
    let review = plan_pull(&checked, live.clone(), false);
    assert!(review.requires_acceptance);
    assert!(review.replacement.is_none());
    assert!(!review.diff.is_empty());
    let accepted = plan_pull(&checked, live.clone(), true);
    assert!(!accepted.requires_acceptance);
    assert_eq!(accepted.replacement, Some(live));
}

#[test]
fn validation_reports_object_diffs_and_affected_queries_without_writes() {
    let canonical = schema(false);
    let changed = schema(true);
    let signatures = signatures();
    let report = validate_schema_authorities(
        &canonical,
        [NamedSchema {
            authority: "live-catalog".to_string(),
            schema: changed,
        }],
        Some(&signatures),
    )
    .expect("validation report");
    assert!(!report.valid);
    assert_eq!(report.comparisons[0].diff.objects.len(), 1);
    assert_eq!(
        report.affected_queries,
        BTreeSet::from(["app::get_user".to_string()])
    );
    assert_eq!(
        affected_queries(
            &signatures,
            &BTreeSet::from([ObjectId::new("public.users.id")])
        ),
        report.affected_queries
    );
}

#[test]
fn authority_conflicts_and_credentials_fail_closed() {
    let first = authority(schema(false));
    let second = authority(schema(true));
    let conflict = resolve_build_authority(
        vec![
            NamedProfileAuthority {
                name: "source".to_string(),
                authority: first.clone(),
            },
            NamedProfileAuthority {
                name: "migration".to_string(),
                authority: second,
            },
        ],
        AuthorityMergeRule::IdenticalSchemas,
    )
    .expect_err("different authorities must fail");
    assert_eq!(
        conflict.kind,
        SchemaLifecycleErrorKind::ConflictingAuthority
    );

    let mut credential = first;
    credential
        .profile
        .schema
        .objects
        .get_mut(&ObjectId::new("public.users"))
        .expect("table")
        .semantic
        .insert(
            "password".to_string(),
            SemanticValue::Text("must-not-appear".to_string()),
        );
    let failure = build_schema_artifacts(&credential).expect_err("credential must fail");
    assert_eq!(failure.kind, SchemaLifecycleErrorKind::CredentialDisclosure);
    assert!(!failure.message.contains("must-not-appear"));
}

fn authority(schema: SchemaIr) -> sifr_sql_contract::ProfileAuthority {
    build_profile_authority(SchemaProfile {
        package_id: "app@1.0.0#path".to_string(),
        name: "app".to_string(),
        source_files: BTreeSet::from(["db/schema.sql".to_string()]),
        source_fingerprints: BTreeMap::from([("db/schema.sql".to_string(), "b".repeat(64))]),
        evidence: SchemaEvidence::Introspection,
        strictness: SchemaStrictness::Exact,
        pooling: PoolingMode::Session,
        session: SessionContract::default(),
        accepted_signers: BTreeSet::new(),
        capabilities: BTreeSet::from(["sql.query.select".to_string()]),
        schema,
    })
    .expect("authority")
}

fn schema(nullable: bool) -> SchemaIr {
    let namespace = SchemaObject {
        identity: ObjectId::new("public"),
        kind: SchemaObjectKind::Namespace,
        semantic: BTreeMap::from([(
            "name".to_string(),
            SemanticValue::Text("public".to_string()),
        )]),
        dependencies: BTreeSet::new(),
        source: None,
    };
    let table = SchemaObject {
        identity: ObjectId::new("public.users"),
        kind: SchemaObjectKind::Table,
        semantic: BTreeMap::from([("name".to_string(), SemanticValue::Text("users".to_string()))]),
        dependencies: BTreeSet::from([ObjectId::new("public")]),
        source: None,
    };
    let column = SchemaObject {
        identity: ObjectId::new("public.users.id"),
        kind: SchemaObjectKind::Column,
        semantic: BTreeMap::from([("nullable".to_string(), SemanticValue::Bool(nullable))]),
        dependencies: BTreeSet::from([ObjectId::new("public.users")]),
        source: None,
    };
    SchemaIr {
        format_version: SCHEMA_IR_FORMAT_VERSION,
        provider: ProviderIdentity {
            package_id: "sifr-sql-postgresql@0.0.0#path".to_string(),
            package_version: Version::new(0, 0, 0),
            package_source: "path+crates/sifr_sql_postgresql".to_string(),
            package_graph_digest: "graph".to_string(),
            compiler_components: BTreeMap::from([("schema".to_string(), "a".repeat(64))]),
        },
        dialect: DialectIdentity {
            family: "postgresql".to_string(),
            server_version: "18".to_string(),
            modes: BTreeSet::new(),
            features: BTreeSet::from(["core-semantics".to_string()]),
        },
        objects: [namespace, table, column]
            .into_iter()
            .map(|object| (object.identity.clone(), object))
            .collect(),
    }
}

fn signatures() -> QuerySignatureArtifact {
    let entry = QuerySignatureEntry {
        module: "app".to_string(),
        symbol: "get_user".to_string(),
        template_identity: "query".to_string(),
        profile_identity: "profile".to_string(),
        schema_fingerprint: "fingerprint".to_string(),
        parameters: Vec::new(),
        result: vec![("id".to_string(), SifrType::ExactInteger)],
        cardinality: Cardinality::AT_MOST_ONE,
        effects: EffectContract::new(
            QueryEffect::Read,
            BTreeSet::from([ObjectId::new("public.users.id")]),
            BTreeSet::new(),
        )
        .expect("effect"),
        schema_dependencies: BTreeSet::from([ObjectId::new("public.users.id")]),
    };
    QuerySignatureArtifact::build("app@1", [entry]).expect("signatures")
}

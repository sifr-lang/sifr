#![allow(clippy::expect_used, clippy::unwrap_used)]

use semver::Version;
use sifr_sql_contract::{
    PoolingMode, ProviderIdentity, SchemaEvidence, SchemaProfile, SchemaRequirement,
    SchemaRequirementIdentity, SchemaStrictness, SessionContract, build_profile_authority,
    build_provider_schema_requirement, normalize_schema, schema_source_fingerprint,
};
use sifr_sql_sqlite::{
    SUPPORTED_SQLITE_SERIES, SqliteAffinity, SqliteConflictForm, SqliteEditorFacts, SqliteParser,
    SqliteSchemaOptions, SqliteStatementKind, affinity, normalize_sqlite_documents,
    sqlite_capabilities,
};
use std::collections::{BTreeMap, BTreeSet};

fn parser() -> SqliteParser {
    SqliteParser::new(SUPPORTED_SQLITE_SERIES[0], BTreeSet::<String>::new())
        .expect("qualified parser")
}

#[test]
fn syntaqlite_grammar_affinity_strict_rowid_and_generated_columns_are_owned() {
    let statements = parser()
        .parse(
            "CREATE TABLE main.events(\
             id INTEGER PRIMARY KEY, payload ANY NOT NULL, \
             slug TEXT GENERATED ALWAYS AS (json_extract(payload, '$.slug')) STORED, \
             UNIQUE(slug)) STRICT;",
        )
        .expect("valid SQLite grammar");
    let SqliteStatementKind::CreateTable(table) = &statements[0].kind else {
        panic!("expected table")
    };
    assert!(table.strict);
    assert!(!table.without_rowid);
    assert!(table.columns[0].auto_increment);
    assert!(
        table.columns[2]
            .generated
            .as_ref()
            .is_some_and(|value| value.stored)
    );
    assert_eq!(affinity("UNSIGNED BIG INT"), SqliteAffinity::Integer);
    assert_eq!(affinity("STRING"), SqliteAffinity::Numeric);
    assert_eq!(affinity(""), SqliteAffinity::Blob);
}

#[test]
fn sqlite_conflict_forms_and_returning_follow_the_pinned_grammar() {
    let statements = parser()
        .parse(
            "INSERT INTO events(id, payload) VALUES (?, ?) \
             ON CONFLICT(id) DO UPDATE SET payload=excluded.payload RETURNING id;",
        )
        .expect("SQLite upsert");
    let SqliteStatementKind::Insert(write) = &statements[0].kind else {
        panic!("expected insert")
    };
    assert_eq!(write.conflict, SqliteConflictForm::UpsertDoUpdate);
    assert_eq!(write.expressions.len(), 2);
}

#[test]
fn schema_identity_contains_version_flags_features_and_attached_scope() {
    let output = normalize_sqlite_documents(
        ProviderIdentity {
            package_id: "sifr-sql-sqlite".to_string(),
            package_version: Version::new(0, 0, 0),
            package_source: "workspace".to_string(),
            package_graph_digest: "a".repeat(64),
            compiler_components: BTreeMap::from([("sqlite".to_string(), "b".repeat(64))]),
        },
        &parser(),
        &SqliteSchemaOptions {
            default_schema: "main".to_string(),
            compile_flags: BTreeSet::new(),
            attached_schemas: BTreeSet::from(["analytics".to_string()]),
            required_features: BTreeSet::from(["json".to_string()]),
            extensions: BTreeSet::new(),
        },
        vec![(
            "db/schema.sql".to_string(),
            "CREATE TABLE users(id INTEGER PRIMARY KEY, email TEXT UNIQUE) STRICT;".to_string(),
        )],
    )
    .expect("normalized SQLite schema");
    assert_eq!(output.dialect.server_version, "3.53.2");
    assert!(output.dialect.features.contains("json"));
    assert!(
        output.documents[0]
            .objects
            .iter()
            .any(|object| object.identity.as_str() == "main.users")
    );
}

#[test]
fn unsupported_versions_flags_and_invalid_sql_are_rejected() {
    assert!(
        SqliteParser::new(
            sifr_sql_sqlite::SqliteServerSeries::new(3, 52, 0),
            BTreeSet::<String>::new(),
        )
        .is_err()
    );
    assert!(SqliteParser::new(SUPPORTED_SQLITE_SERIES[0], ["SQLITE_OMIT_JSON"],).is_err());
    assert!(parser().parse("SELECT FROM users").is_err());
}

#[test]
fn sqlite_editor_recovery_is_non_authoritative_and_provider_documented() {
    let facts = SqliteEditorFacts::analyze(&parser(), "SELECT id FROM users WHERE");
    assert!(!facts.recovery.compile_authority);
    assert!(facts.normalized.is_none());
    assert_eq!(facts.documentation_base, "https://sqlite.org/lang.html");
    assert!(
        facts
            .completion_keywords
            .contains(&"ON CONFLICT DO UPDATE".to_string())
    );
    assert_eq!(facts.semantic_settings_fingerprint.len(), 64);
}

#[test]
fn sqlite_independently_normalizes_proves_and_specializes_portable_requirements() {
    let requirement_source =
        "CREATE TABLE users(id INTEGER PRIMARY KEY, email TEXT NOT NULL, UNIQUE(email)) STRICT";
    let requirement = normalize_sqlite_documents(
        provider(),
        &parser(),
        &schema_options(),
        vec![(
            "requirements/has_users.sqlite.sql".to_string(),
            requirement_source.to_string(),
        )],
    )
    .expect("requirement normalization");
    let requirement_schema =
        normalize_schema(provider(), requirement.dialect, requirement.documents)
            .expect("requirement schema");
    let identity = SchemaRequirementIdentity::new("library", "has_users").expect("identity");
    let artifact = build_provider_schema_requirement(
        identity.clone(),
        "requirements/has_users.sqlite.sql",
        schema_source_fingerprint(requirement_source.as_bytes()),
        &requirement_schema,
        BTreeSet::from(["sql.query.select".to_string()]),
        &sqlite_capabilities(),
    )
    .expect("requirement artifact");

    let application_source = "CREATE TABLE users(\
        id INTEGER PRIMARY KEY, email TEXT NOT NULL, display_name TEXT,\
        UNIQUE(email), CHECK(display_name <> '')) STRICT";
    let application = normalize_sqlite_documents(
        provider(),
        &parser(),
        &schema_options(),
        vec![("db/schema.sql".to_string(), application_source.to_string())],
    )
    .expect("application normalization");
    let application_schema =
        normalize_schema(provider(), application.dialect, application.documents)
            .expect("application schema");
    let authority = build_profile_authority(SchemaProfile {
        package_id: "app@1.0.0#registry".to_string(),
        name: "app".to_string(),
        source_files: BTreeSet::from(["db/schema.sql".to_string()]),
        source_fingerprints: BTreeMap::from([(
            "db/schema.sql".to_string(),
            schema_source_fingerprint(application_source.as_bytes()),
        )]),
        evidence: SchemaEvidence::MigrationHead,
        strictness: SchemaStrictness::Compatible,
        pooling: PoolingMode::Session,
        session: SessionContract {
            search_path: vec!["main".to_string()],
            ..SessionContract::default()
        },
        accepted_signers: BTreeSet::new(),
        capabilities: sqlite_capabilities(),
        schema: application_schema,
    })
    .expect("authority");
    SchemaRequirement::new(identity, [artifact])
        .expect("requirement")
        .prove(&authority)
        .expect("structural proof");
}

fn provider() -> ProviderIdentity {
    ProviderIdentity {
        package_id: "sifr-sql-sqlite".to_string(),
        package_version: Version::new(0, 0, 0),
        package_source: "workspace".to_string(),
        package_graph_digest: "a".repeat(64),
        compiler_components: BTreeMap::from([("sqlite".to_string(), "b".repeat(64))]),
    }
}

fn schema_options() -> SqliteSchemaOptions {
    SqliteSchemaOptions {
        default_schema: "main".to_string(),
        compile_flags: BTreeSet::new(),
        attached_schemas: BTreeSet::new(),
        required_features: BTreeSet::from(["json".to_string()]),
        extensions: BTreeSet::new(),
    }
}

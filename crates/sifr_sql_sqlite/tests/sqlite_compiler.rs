#![allow(clippy::expect_used, clippy::unwrap_used)]

use semver::Version;
use sifr_sql_contract::{
    Cardinality, PoolingMode, ProviderIdentity, SchemaDocumentKind, SchemaEvidence,
    SchemaObjectKind, SchemaProfile, SchemaRequirement, SchemaRequirementIdentity,
    SchemaSourceInput, SchemaStrictness, SessionContract, build_profile_authority,
    build_provider_schema_requirement, normalize_schema, project_provider_requirement_schema,
    schema_normalization_from_response, schema_normalization_request, schema_source_fingerprint,
};
use sifr_sql_sqlite::{
    SUPPORTED_SQLITE_SERIES, SqliteAffinity, SqliteAnalyzer, SqliteConflictForm, SqliteEditorFacts,
    SqliteParser, SqliteSchemaOptions, SqliteStatementKind, affinity, component_registration,
    execute_embedded_request, normalize_sqlite_documents, sqlite_capabilities,
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
    assert!(table.columns[0].primary_key);
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
fn sqlite_parameter_slots_limits_returning_conflicts_and_operators_are_exact() {
    let schema = normalized_schema(
        "CREATE TABLE users(id INTEGER PRIMARY KEY, email TEXT NOT NULL) STRICT;",
    );
    let selected_parser = parser();
    let analyzer = SqliteAnalyzer::new(&selected_parser, &schema).expect("analyzer");
    let analysis = analyzer
        .analyze_query(
            "SELECT id FROM users WHERE id=?1 OR id=?1 OR email=:email OR email=:email OR id=? LIMIT 1, 5",
        )
        .expect("parameter query");
    assert_eq!(analysis.parameters.len(), 3);
    assert_eq!(analysis.cardinality, Cardinality::MANY);
    assert!(analysis.normalized_statement.contains("?1"));
    assert!(analysis.normalized_statement.contains(":email"));

    let one = analyzer
        .analyze_query("SELECT id FROM users LIMIT 5, 1")
        .expect("comma limit");
    assert_eq!(one.cardinality, Cardinality::AT_MOST_ONE);
    let explicit_gap = analyzer
        .analyze_query("SELECT id FROM users WHERE id=?5 OR email=$name OR email=$name")
        .expect("explicit and repeated slots");
    assert_eq!(explicit_gap.parameters.len(), 6);
    assert!(
        analyzer
            .analyze_query("SELECT id FROM users WHERE id=?0")
            .is_err()
    );

    let returning = analyzer
        .analyze_query("UPDATE OR FAIL users SET email=$email WHERE id=$id RETURNING id, email")
        .expect("returning write");
    assert_eq!(returning.parameters.len(), 2);
    assert_eq!(returning.result_fields.len(), 2);
    assert_eq!(returning.cardinality, Cardinality::MANY);
    assert!(
        returning
            .required_capabilities
            .contains("sql.write.returning")
    );
    assert!(
        returning
            .required_capabilities
            .contains("sql.sqlite.write.conflict")
    );

    for source in [
        "INSERT OR REPLACE INTO users(id, email) VALUES (1, 'a')",
        "INSERT OR IGNORE INTO users(id, email) VALUES (1, 'a')",
        "INSERT OR ROLLBACK INTO users(id, email) VALUES (1, 'a')",
        "INSERT OR ABORT INTO users(id, email) VALUES (1, 'a')",
        "INSERT OR FAIL INTO users(id, email) VALUES (1, 'a')",
        "REPLACE INTO users(id, email) VALUES (1, 'a')",
    ] {
        let analysis = analyzer.analyze_query(source).expect(source);
        assert!(
            analysis
                .required_capabilities
                .contains("sql.sqlite.write.conflict"),
            "{source}"
        );
    }
    parser()
        .parse("SELECT (1 | 2) & ~3, '{}' -> 'x', '{}' ->> 'x'")
        .expect("SQLite operators");
}

#[test]
fn sqlite_rowid_untyped_keyword_columns_and_drop_forms_are_exact() {
    let schema = normalized_schema(
        "CREATE TABLE inline_alias(id INTEGER PRIMARY KEY);
         CREATE TABLE inline_desc(id INTEGER PRIMARY KEY DESC);
         CREATE TABLE no_rowid(id INTEGER PRIMARY KEY) WITHOUT ROWID;
         CREATE TABLE table_alias(id INTEGER, PRIMARY KEY(id));
         CREATE TABLE composite(a INTEGER, b INTEGER, PRIMARY KEY(a, b));
         CREATE TABLE auto_table(id INTEGER PRIMARY KEY AUTOINCREMENT);
         CREATE TABLE loose(value, key TEXT, \"index\" INTEGER);",
    );
    for identity in [
        "main.inline_alias.id",
        "main.table_alias.id",
        "main.auto_table.id",
    ] {
        assert_eq!(
            schema.objects[&sifr_sql_contract::ObjectId::new(identity)].kind,
            SchemaObjectKind::IdentityColumn
        );
    }
    for identity in [
        "main.inline_desc.id",
        "main.no_rowid.id",
        "main.composite.a",
        "main.composite.b",
        "main.loose.value",
    ] {
        assert_eq!(
            schema.objects[&sifr_sql_contract::ObjectId::new(identity)].kind,
            SchemaObjectKind::Column
        );
    }
    assert!(
        normalize_sqlite_documents(
            provider(),
            &parser(),
            &schema_options(),
            vec![(
                "db/invalid.sql".to_string(),
                "CREATE TABLE invalid(id INT PRIMARY KEY AUTOINCREMENT)".to_string(),
            )],
        )
        .is_err()
    );
    assert!(
        normalize_sqlite_documents(
            provider(),
            &parser(),
            &schema_options(),
            vec![(
                "db/strict.sql".to_string(),
                "CREATE TABLE invalid(value) STRICT".to_string(),
            )],
        )
        .is_err()
    );
    let selected_parser = parser();
    let analyzer = SqliteAnalyzer::new(&selected_parser, &schema).expect("analyzer");
    analyzer
        .analyze_query("SELECT key, \"index\" FROM loose")
        .expect("keyword-named columns");
    for ddl in [
        "DROP TABLE IF EXISTS loose",
        "DROP INDEX IF EXISTS loose_index",
        "DROP VIEW IF EXISTS loose_view",
        "DROP TRIGGER IF EXISTS loose_trigger",
    ] {
        assert!(matches!(
            parser().parse(ddl).expect(ddl)[0].kind,
            SqliteStatementKind::Drop(_)
        ));
    }
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
        output.documents[1]
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
    let requirement_schema = project_provider_requirement_schema(
        &requirement_schema,
        "requirements/has_users.sqlite.sql",
    )
    .expect("requirement projection");
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

#[test]
fn embedded_normalization_keeps_metadata_separate_and_builds_a_requirement() {
    let source_text = "CREATE TABLE users(id INTEGER PRIMARY KEY, email TEXT NOT NULL) STRICT";
    let source = SchemaSourceInput {
        document: "requirements/embedded.sqlite.sql".to_string(),
        kind: SchemaDocumentKind::SqlDdl,
        fingerprint: schema_source_fingerprint(source_text.as_bytes()),
        contents: source_text.as_bytes().to_vec(),
    };
    let mut registration =
        component_registration(SUPPORTED_SQLITE_SERIES[0]).expect("registration");
    registration.identity.processor = "sifr.sql.sqlite.schema".to_string();
    let request = schema_normalization_request(
        &registration,
        "0.0.0",
        "portable::embedded::sqlite",
        "3.53.2",
        &SessionContract {
            search_path: vec!["main".to_string()],
            ..SessionContract::default()
        },
        &BTreeSet::new(),
        std::slice::from_ref(&source),
    )
    .expect("normalization request");
    let response = execute_embedded_request(request).expect("embedded response");
    let normalized =
        schema_normalization_from_response(provider(), std::slice::from_ref(&source), &response)
            .expect("normalized response");
    let projected = project_provider_requirement_schema(&normalized.schema, &source.document)
        .expect("requirement projection");
    build_provider_schema_requirement(
        SchemaRequirementIdentity::new("library", "embedded").expect("identity"),
        source.document,
        source.fingerprint,
        &projected,
        BTreeSet::from(["sql.query.select".to_string()]),
        &normalized.capabilities,
    )
    .expect("requirement artifact");
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

fn normalized_schema(source: &str) -> sifr_sql_contract::SchemaIr {
    let output = normalize_sqlite_documents(
        provider(),
        &parser(),
        &schema_options(),
        vec![("db/schema.sql".to_string(), source.to_string())],
    )
    .expect("SQLite schema normalization");
    normalize_schema(provider(), output.dialect, output.documents).expect("canonical schema")
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

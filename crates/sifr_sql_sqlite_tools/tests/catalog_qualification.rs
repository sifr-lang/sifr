#![allow(clippy::expect_used)]

use rusqlite::Connection;
use semver::Version;
use sifr_sql_contract::{
    DialectIdentity, ProviderIdentity, SchemaObjectKind, normalize_schema, semantic_diff,
};
use sifr_sql_sqlite::{
    SUPPORTED_SQLITE_SERIES, SqliteParser, SqliteSchemaOptions, normalize_sqlite_documents,
};
use sifr_sql_sqlite_tools::pull_live_catalog_from_connection;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn catalog_reflects_strict_affinity_rowid_generated_indexes_and_triggers() {
    let connection = Connection::open_in_memory().expect("open");
    let ddl = "CREATE TABLE events(
                id INTEGER PRIMARY KEY,
                payload TEXT NOT NULL,
                slug TEXT GENERATED ALWAYS AS (json_extract(payload, '$.slug')) STORED
             ) STRICT;
             CREATE UNIQUE INDEX events_slug ON events(slug);
             CREATE TRIGGER events_no_delete BEFORE DELETE ON events BEGIN SELECT RAISE(ABORT, 'no'); END;
             CREATE TRIGGER events_payload BEFORE INSERT ON events BEGIN
                 SELECT CASE WHEN NEW.payload = '' THEN RAISE(ABORT, 'empty') ELSE NEW.payload END;
                 SELECT 1;
             END;";
    connection.execute_batch(ddl).expect("schema");
    let provider = provider();
    let dialect = dialect();
    let schema = pull_live_catalog_from_connection(&connection, provider.clone(), dialect.clone())
        .expect("catalog");
    assert_eq!(
        schema.objects[&sifr_sql_contract::ObjectId::new("main.events")].kind,
        SchemaObjectKind::Table
    );
    assert_eq!(
        schema.objects[&sifr_sql_contract::ObjectId::new("main.events.id")].kind,
        SchemaObjectKind::IdentityColumn
    );
    assert!(
        schema
            .objects
            .values()
            .any(|object| object.kind == SchemaObjectKind::Trigger)
    );
    assert!(
        schema
            .objects
            .values()
            .any(|object| object.kind == SchemaObjectKind::Index)
    );
    let parser =
        SqliteParser::new(SUPPORTED_SQLITE_SERIES[0], BTreeSet::<String>::new()).expect("parser");
    let expected = normalize_sqlite_documents(
        provider.clone(),
        &parser,
        &SqliteSchemaOptions {
            default_schema: "main".to_string(),
            compile_flags: BTreeSet::new(),
            attached_schemas: BTreeSet::new(),
            required_features: dialect.features.clone(),
            extensions: BTreeSet::new(),
        },
        vec![("db/schema.sql".to_string(), ddl.to_string())],
    )
    .expect("DDL normalization");
    let expected =
        normalize_schema(provider, expected.dialect, expected.documents).expect("expected schema");
    assert!(semantic_diff(&expected, &schema).is_empty());
}

#[test]
fn catalog_reflects_explicit_attached_namespaces() {
    let connection = Connection::open_in_memory().expect("open");
    connection
        .execute_batch(
            "ATTACH DATABASE ':memory:' AS analytics;
             CREATE TABLE analytics.events(id INTEGER PRIMARY KEY) STRICT;",
        )
        .expect("attached schema");
    let schema = pull_live_catalog_from_connection(&connection, provider(), dialect())
        .expect("attached catalog");
    assert_eq!(
        schema.objects[&sifr_sql_contract::ObjectId::new("analytics")].kind,
        SchemaObjectKind::Namespace
    );
    assert_eq!(
        schema.objects[&sifr_sql_contract::ObjectId::new("analytics.events")].kind,
        SchemaObjectKind::Table
    );
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

fn dialect() -> DialectIdentity {
    DialectIdentity {
        family: "sqlite".to_string(),
        server_version: "3.53.2".to_string(),
        modes: BTreeSet::new(),
        features: BTreeSet::from(["json".to_string()]),
    }
}

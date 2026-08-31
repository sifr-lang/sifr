#![allow(clippy::expect_used)]

use rusqlite::Connection;
use semver::Version;
use sifr_sql_contract::{DialectIdentity, ProviderIdentity, SchemaObjectKind};
use sifr_sql_sqlite_tools::pull_live_catalog_from_connection;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn catalog_reflects_strict_affinity_rowid_generated_indexes_and_triggers() {
    let connection = Connection::open_in_memory().expect("open");
    connection
        .execute_batch(
            "CREATE TABLE events(
                id INTEGER PRIMARY KEY,
                payload TEXT NOT NULL,
                slug TEXT GENERATED ALWAYS AS (json_extract(payload, '$.slug')) STORED
             ) STRICT;
             CREATE UNIQUE INDEX events_slug ON events(slug);
             CREATE TRIGGER events_no_delete BEFORE DELETE ON events BEGIN SELECT RAISE(ABORT, 'no'); END;",
        )
        .expect("schema");
    let schema = pull_live_catalog_from_connection(
        &connection,
        ProviderIdentity {
            package_id: "sifr-sql-sqlite".to_string(),
            package_version: Version::new(0, 0, 0),
            package_source: "workspace".to_string(),
            package_graph_digest: "a".repeat(64),
            compiler_components: BTreeMap::from([("sqlite".to_string(), "b".repeat(64))]),
        },
        DialectIdentity {
            family: "sqlite".to_string(),
            server_version: "3.53.2".to_string(),
            modes: BTreeSet::new(),
            features: BTreeSet::from(["json".to_string()]),
        },
    )
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
}

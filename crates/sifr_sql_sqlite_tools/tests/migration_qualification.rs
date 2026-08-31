#![allow(clippy::expect_used)]

use rusqlite::Connection;
use sifr_sql_sqlite_tools::{
    SqliteMigrationAction, SqliteMigrationPlan, connect_migration_runtime,
    validate_sqlite_migration_plan,
};

#[test]
fn rebuild_preserves_rows_constraints_and_recreates_owned_objects() {
    let directory = tempfile::tempdir().expect("directory");
    let path = directory.path().join("migration.sqlite3");
    let connection = Connection::open(&path).expect("open");
    connection
        .execute_batch(
            "PRAGMA foreign_keys=ON;
             CREATE TABLE parents(id INTEGER PRIMARY KEY) STRICT;
             CREATE TABLE items(id INTEGER PRIMARY KEY, parent_id INTEGER REFERENCES parents(id), value TEXT) STRICT;
             INSERT INTO parents VALUES (1);
             INSERT INTO items VALUES (7, 1, 'kept');",
        )
        .expect("fixture");
    drop(connection);
    let plan = SqliteMigrationPlan {
        schema_fingerprint_before: "a".repeat(64),
        schema_fingerprint_after: "b".repeat(64),
        actions: vec![SqliteMigrationAction::RebuildTable {
            table: "items".to_string(),
            temporary_table: "__sifr_rebuild_items".to_string(),
            create_temporary_sql: "CREATE TABLE __sifr_rebuild_items(id INTEGER PRIMARY KEY, parent_id INTEGER REFERENCES parents(id), value TEXT NOT NULL, note TEXT) STRICT".to_string(),
            target_columns: vec!["id".to_string(), "parent_id".to_string(), "value".to_string(), "note".to_string()],
            source_expressions: vec!["id".to_string(), "parent_id".to_string(), "value".to_string(), "'new'".to_string()],
            recreate_sql: vec!["CREATE INDEX items_value ON items(value)".to_string()],
        }],
    };
    let mut runtime = connect_migration_runtime(&path).expect("runtime");
    runtime.apply(&plan).expect("apply rebuild");
    let connection = Connection::open(&path).expect("reopen");
    let row: (String, String) = connection
        .query_row("SELECT value, note FROM items WHERE id=7", [], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .expect("row");
    assert_eq!(row, ("kept".to_string(), "new".to_string()));
    let violations: i64 = connection
        .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .expect("foreign key check");
    assert_eq!(violations, 0);
}

#[test]
fn migration_validation_rejects_unowned_scope_and_ambiguous_rebuilds() {
    let mut plan = SqliteMigrationPlan {
        schema_fingerprint_before: "a".repeat(64),
        schema_fingerprint_after: "b".repeat(64),
        actions: vec![SqliteMigrationAction::Statement {
            sql: "ATTACH 'other.db' AS other".to_string(),
        }],
    };
    assert!(validate_sqlite_migration_plan(&plan).is_err());
    plan.actions = vec![SqliteMigrationAction::RebuildTable {
        table: "items".to_string(),
        temporary_table: "items_new".to_string(),
        create_temporary_sql: "CREATE TABLE items_new(id INTEGER)".to_string(),
        target_columns: vec!["id".to_string()],
        source_expressions: vec![],
        recreate_sql: vec![],
    }];
    assert!(validate_sqlite_migration_plan(&plan).is_err());
}

#![allow(clippy::expect_used)]

use sifr_sql_sqlite::{SUPPORTED_SQLITE_SERIES, SqliteParser};
use std::collections::BTreeSet;

#[test]
fn normalization_is_idempotent_for_qualified_sqlite_corpus() {
    let parser =
        SqliteParser::new(SUPPORTED_SQLITE_SERIES[0], BTreeSet::<String>::new()).expect("parser");
    for source in [
        "SELECT id, payload FROM events WHERE id = ? ORDER BY id LIMIT 10",
        "WITH active AS (SELECT id FROM users WHERE enabled = 1) SELECT id FROM active",
        "INSERT OR IGNORE INTO users(id, email) VALUES (?, ?)",
        "UPDATE users SET email = ? WHERE id = ? RETURNING id",
        "DELETE FROM users WHERE id = ? RETURNING id",
        "CREATE TABLE items(id INTEGER PRIMARY KEY, value TEXT) WITHOUT ROWID",
    ] {
        let first = parser.normalize(source).expect(source);
        let second = parser.normalize(&first).expect(&first);
        assert_eq!(first, second);
    }
}

#[test]
fn malformed_inputs_never_escape_the_diagnostic_boundary() {
    let parser =
        SqliteParser::new(SUPPORTED_SQLITE_SERIES[0], BTreeSet::<String>::new()).expect("parser");
    for bytes in 0_u8..=255 {
        let source = String::from_utf8_lossy(&[bytes, b'(', b'?', b')']).into_owned();
        let outcome = std::panic::catch_unwind(|| parser.parse(&source));
        assert!(outcome.is_ok());
    }
}

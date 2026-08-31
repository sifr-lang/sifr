#![allow(clippy::expect_used)]

use sifr_sql_mysql::{MysqlParser, MysqlServerSeries};
use std::time::{Duration, Instant};

#[test]
fn parser_and_recovery_do_not_panic_on_protocol_corpus() {
    let parser = MysqlParser::new(
        MysqlServerSeries::new(26, 7),
        ["ANSI_QUOTES"],
        "utf8mb4_0900_ai_ci",
    )
    .expect("parser");
    let atoms = [
        "",
        "'",
        "`",
        "/*",
        "?",
        "(",
        ")",
        "SELECT",
        "CREATE TABLE",
        "\0",
    ];
    for left in atoms {
        for right in atoms {
            let input = format!("{left} {right}");
            let _parsed = parser.parse(&input);
            let recovery = sifr_sql_mysql::recover_document(&input);
            assert!(!recovery.compile_authority);
        }
    }
}

#[test]
fn normalization_is_stable_for_whitespace_and_comments() {
    let parser = MysqlParser::new(
        MysqlServerSeries::new(8, 4),
        std::iter::empty::<String>(),
        "utf8mb4_0900_ai_ci",
    )
    .expect("parser");
    assert_eq!(
        parser
            .normalize("SELECT id FROM users WHERE id=?")
            .expect("first"),
        parser
            .normalize(" /* lead */ SELECT  id\nFROM users WHERE id = ? -- tail\n")
            .expect("second")
    );
}

#[test]
fn warm_parser_batch_stays_within_the_named_editor_budget() {
    const BUDGET: Duration = Duration::from_secs(2);
    let parser = MysqlParser::new(
        MysqlServerSeries::new(8, 4),
        ["STRICT_TRANS_TABLES"],
        "utf8mb4_0900_ai_ci",
    )
    .expect("parser");
    let statement = "SELECT u.id, u.email FROM users AS u WHERE u.id = ? AND u.email IS NOT NULL ORDER BY u.id LIMIT 25";
    parser.parse(statement).expect("warm parser");
    let started = Instant::now();
    for _ in 0..1_000 {
        parser.parse(statement).expect("budget parse");
    }
    assert!(
        started.elapsed() <= BUDGET,
        "warm MySQL parser batch exceeded {BUDGET:?}"
    );
}

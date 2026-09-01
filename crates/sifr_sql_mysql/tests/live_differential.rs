#![allow(clippy::expect_used)]

use mysql_async::{Conn, Opts, prelude::Queryable};
use sifr_sql_mysql::{MysqlParser, MysqlServerSeries};

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires SIFR_MYSQL_TEST_URL"]
async fn parser_acceptance_matches_supported_mysql_server() {
    let url = std::env::var("SIFR_MYSQL_TEST_URL").expect("URL");
    let series = std::env::var("SIFR_MYSQL_TEST_SERIES").expect("series");
    let (major, minor) = series.split_once('.').expect("series pair");
    let mut connection = Conn::new(Opts::from_url(&url).expect("opts"))
        .await
        .expect("connection");
    let modes: String = connection
        .query_first("SELECT @@session.sql_mode")
        .await
        .expect("modes")
        .expect("mode row");
    let collation: String = connection
        .query_first("SELECT @@collation_connection")
        .await
        .expect("collation")
        .expect("collation row");
    let character_set: String = connection
        .query_first("SELECT @@character_set_connection")
        .await
        .expect("character set")
        .expect("character-set row");
    let parser = MysqlParser::new(
        MysqlServerSeries::new(major.parse().expect("major"), minor.parse().expect("minor")),
        modes.split(',').filter(|mode| !mode.is_empty()),
        character_set,
        collation,
    )
    .expect("parser");
    for statement in [
        "SELECT id, email FROM users WHERE id = ? LIMIT 1",
        "INSERT INTO users(id, email) VALUES (?, ?) ON DUPLICATE KEY UPDATE email = ?",
        "UPDATE users SET email = ? WHERE id = ?",
        "DELETE FROM users WHERE id = ?",
    ] {
        assert!(
            parser.parse(statement).is_ok(),
            "provider rejected {statement}"
        );
        let escaped = statement.replace('\\', "\\\\").replace('\'', "\\'");
        connection
            .query_drop(format!("PREPARE sifr_differential FROM '{escaped}'"))
            .await
            .expect("server parser");
        connection
            .query_drop("DEALLOCATE PREPARE sifr_differential")
            .await
            .expect("deallocate");
    }
    let invalid = "SELECT FROM users";
    assert!(parser.parse(invalid).is_err());
    assert!(
        connection
            .query_drop(format!("PREPARE sifr_invalid FROM '{invalid}'"))
            .await
            .is_err()
    );
}

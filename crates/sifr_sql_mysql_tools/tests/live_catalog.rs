#![allow(clippy::expect_used)]

use mysql_async::{Conn, Opts, prelude::Queryable};
use semver::Version;
use sifr_sql_contract::{DialectIdentity, ProviderIdentity, SchemaObjectKind};
use sifr_sql_mysql_tools::pull_live_catalog;
use std::collections::{BTreeMap, BTreeSet};

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires SIFR_MYSQL_TEST_URL"]
async fn live_catalog_preserves_mysql_schema_semantics() {
    let url = std::env::var("SIFR_MYSQL_TEST_URL").expect("URL");
    let series = std::env::var("SIFR_MYSQL_TEST_SERIES").expect("series");
    let mut connection = Conn::new(Opts::from_url(&url).expect("opts"))
        .await
        .expect("connection");
    let metadata: (String, String, String) = connection
        .query_first("SELECT @@session.sql_mode, @@character_set_database, @@collation_database")
        .await
        .expect("settings")
        .expect("settings row");
    connection.disconnect().await.expect("disconnect");
    let modes = metadata
        .0
        .split(',')
        .filter(|mode| !mode.is_empty())
        .map(str::to_string)
        .chain([
            format!("character-set:{}", metadata.1),
            format!("collation:{}", metadata.2),
        ])
        .collect();
    let schema = pull_live_catalog(
        &url,
        ProviderIdentity {
            package_id: "sifr-sql-mysql@0.0.0#live".to_string(),
            package_version: Version::new(0, 0, 0),
            package_source: "workspace".to_string(),
            package_graph_digest: "a".repeat(64),
            compiler_components: BTreeMap::from([("mysql".to_string(), "b".repeat(64))]),
        },
        DialectIdentity {
            family: "mysql".to_string(),
            server_version: series,
            modes,
            features: BTreeSet::new(),
        },
    )
    .await
    .expect("live schema");
    assert!(
        schema
            .objects
            .values()
            .any(|object| object.kind == SchemaObjectKind::Table)
    );
    assert!(
        schema
            .objects
            .values()
            .any(|object| object.kind == SchemaObjectKind::Column)
    );
    assert!(
        schema
            .objects
            .values()
            .any(|object| object.kind == SchemaObjectKind::PrimaryKey)
    );
}

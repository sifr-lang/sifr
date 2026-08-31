#![allow(clippy::expect_used)]

use semver::Version;
use sifr_sql_contract::{DialectIdentity, ProviderIdentity, SchemaObjectKind};
use sifr_sql_postgresql_tools::pull_live_catalog;
use std::collections::{BTreeMap, BTreeSet};

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires SIFR_POSTGRESQL_SCHEMA_TOOL_TEST_URL"]
async fn live_catalog_preserves_postgresql_semantic_objects() {
    let url = std::env::var("SIFR_POSTGRESQL_SCHEMA_TOOL_TEST_URL").expect("test URL");
    let major = std::env::var("SIFR_POSTGRESQL_SCHEMA_TOOL_TEST_MAJOR")
        .expect("test major")
        .parse::<u16>()
        .expect("numeric major");
    let dialect = DialectIdentity {
        family: "postgresql".to_string(),
        server_version: major.to_string(),
        modes: BTreeSet::new(),
        features: BTreeSet::from(["core-semantics".to_string()]),
    };
    let schema = pull_live_catalog(&url, provider(), dialect)
        .await
        .expect("live catalog");
    let kinds = schema
        .objects
        .values()
        .map(|object| object.kind)
        .collect::<BTreeSet<_>>();
    let expected = BTreeSet::from([
        SchemaObjectKind::Namespace,
        SchemaObjectKind::Table,
        SchemaObjectKind::Column,
        SchemaObjectKind::PrimaryKey,
        SchemaObjectKind::ForeignKey,
        SchemaObjectKind::CheckConstraint,
        SchemaObjectKind::Index,
        SchemaObjectKind::Sequence,
        SchemaObjectKind::IdentityColumn,
        SchemaObjectKind::View,
        SchemaObjectKind::MaterializedView,
        SchemaObjectKind::Enum,
        SchemaObjectKind::Domain,
        SchemaObjectKind::Composite,
        SchemaObjectKind::Array,
        SchemaObjectKind::Range,
        SchemaObjectKind::Function,
        SchemaObjectKind::Operator,
        SchemaObjectKind::Collation,
        SchemaObjectKind::Extension,
        SchemaObjectKind::Trigger,
        SchemaObjectKind::ServerCapability,
        SchemaObjectKind::DialectMetadata,
    ]);
    assert_eq!(kinds.intersection(&expected).count(), expected.len());
    assert_eq!(schema.dialect.server_version, major.to_string());
    assert!(
        schema
            .objects
            .values()
            .all(|object| !object.semantic.is_empty())
    );
}

fn provider() -> ProviderIdentity {
    ProviderIdentity {
        package_id: "sifr-sql-postgresql@0.0.0#qualification".to_string(),
        package_version: Version::new(0, 0, 0),
        package_source: "path+crates/sifr_sql_postgresql".to_string(),
        package_graph_digest: "qualification-graph".to_string(),
        compiler_components: BTreeMap::from([("schema".to_string(), "a".repeat(64))]),
    }
}

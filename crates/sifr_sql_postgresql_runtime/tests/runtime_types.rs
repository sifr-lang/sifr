#![allow(clippy::expect_used)]

use sifr_sql_postgresql_runtime::{
    PostgresConnection, PostgresPool, PostgresProfile, PostgresRowStream, PostgresTls,
    PostgresTransaction, PostgresTransactionRowStream, Unverified,
};
use sifr_sql_runtime::{
    IsolationLevel, PoolingMode, RuntimeLimits, SchemaDependencySlice, SchemaProperty,
    SchemaStrictness, SessionContract,
};
use static_assertions::{assert_impl_all, assert_not_impl_any};

assert_impl_all!(PostgresPool<Unverified>: Clone, Send, Sync);
assert_not_impl_any!(PostgresConnection: Clone, Send, Sync);
assert_not_impl_any!(PostgresTransaction: Clone, Send, Sync);
assert_not_impl_any!(PostgresRowStream: Clone, Send, Sync);
assert_not_impl_any!(PostgresTransactionRowStream<'static>: Clone, Send, Sync);

#[test]
fn public_configuration_debug_output_is_redacted() {
    let fingerprint = "a".repeat(64);
    let schema = SchemaDependencySlice::new(
        fingerprint.clone(),
        [SchemaProperty::new("probe", Some("present".to_string())).expect("property")],
    )
    .expect("schema");
    let profile = PostgresProfile::new(
        "postgresql://user:super-secret@private.example/app",
        "b".repeat(64),
        schema,
        sifr_sql_postgresql_runtime::PostgresEvidence::Introspection {
            fingerprint_statement: "SELECT repeat('a', 64)".to_string(),
            probes: vec![
                sifr_sql_postgresql_runtime::VerificationProbe::new("probe", "SELECT 'present'")
                    .expect("probe"),
            ],
        },
        SchemaStrictness::Exact,
        SessionContract {
            search_path: vec!["public".to_string()],
            time_zone: "UTC".to_string(),
            role: None,
            default_isolation: IsolationLevel::ReadCommitted,
            read_only: false,
            pooling: PoolingMode::Session,
            requires_session_affinity: false,
        },
        RuntimeLimits::default(),
        PostgresTls::Disabled,
    )
    .expect("profile");
    let rendered = format!("{profile:?}");
    assert!(!rendered.contains("super-secret"));
    assert!(!rendered.contains("private.example"));
    assert!(rendered.contains("<redacted>"));
}

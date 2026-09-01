#![allow(clippy::expect_used, clippy::panic)]

use semver::Version;
use sifr_sql_contract::{
    DialectIdentity, PoolingMode, ProviderIdentity, SchemaEvidence, SchemaObjectKind,
    SchemaProfile, SchemaStrictness, SessionContract, build_profile_authority, normalize_schema,
    semantic_diff,
};
use sifr_sql_postgresql::{
    LibpgQueryParser, PostgresAnalyzer, PostgresCatalog, PostgresCompilerComponent,
    PostgresComponentRequest, PostgresComponentResponse, PostgresTypeRegistry,
};
use sifr_sql_postgresql_tools::pull_live_catalog;
use sifr_sql_tool::{GENERATED_MODULE_PATH, build_schema_artifacts};
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
        features: BTreeSet::from(["core-semantics".to_string(), "libpg-query".to_string()]),
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
            .contains_key(&sifr_sql_contract::ObjectId::new("public.audit_sequence"))
    );
    let owned_sequence = schema
        .objects
        .get(&sifr_sql_contract::ObjectId::new(
            "public.owned_accounts_sequence",
        ))
        .expect("explicit owned sequence");
    assert_eq!(
        owned_sequence.semantic.get("owned-by"),
        Some(&sifr_sql_contract::SemanticValue::Text(
            "public.accounts.id".to_string()
        ))
    );
    assert!(
        owned_sequence
            .dependencies
            .contains(&sifr_sql_contract::ObjectId::new("public.accounts.id"))
    );
    assert!(
        !schema
            .objects
            .contains_key(&sifr_sql_contract::ObjectId::new("public.accounts_id_seq"))
    );
    let catalog = PostgresCatalog::from_schema(&schema, PostgresTypeRegistry::new(major))
        .expect("pulled schema must load in the compiler catalog");
    let analyzer = PostgresAnalyzer::new(LibpgQueryParser, catalog);
    analyzer
        .analyze_query(
            "SELECT add_one(1::integer) AS narrow, add_one(1::bigint) AS wide, 1 === 1 AS equal",
        )
        .expect("overloaded functions and operator must resolve from canonical identities");
    analyzer
        .analyze_query("SELECT address::text AS rendered FROM type_samples")
        .expect("user-defined cast must resolve from the canonical cast account");
    let price_window = schema
        .objects
        .get(&sifr_sql_contract::ObjectId::new("public.price_window"))
        .expect("domain over range");
    assert!(
        price_window.semantic.contains_key("sifr_type"),
        "domain over range semantics: {:?}",
        price_window.semantic
    );
    let artifacts = build_schema_artifacts(&authority(schema.clone())).expect("schema artifacts");
    let generated = String::from_utf8(artifacts.files()[GENERATED_MODULE_PATH].clone())
        .expect("generated source");
    assert!(generated.contains("class domains__public__positive_id:"));
    assert!(generated.contains("value: int32"));
    assert!(generated.contains("unit_count: int32"));
    assert!(generated.contains("latitude: Numeric"));
    assert!(
        generated.contains("domain_values: SqlArray[domains__public__positive_id | None]"),
        "{generated}"
    );
    assert!(
        generated.contains("composite_values: SqlArray[composites__public__postal_address | None]")
    );
    assert!(generated.contains("value: Range[Numeric]"));
    if major == 18 {
        let live = parity_schema(schema);
        let ddl = ddl_parity_schema(provider());
        build_schema_artifacts(&authority(ddl.clone())).expect("DDL schema artifacts");
        let differences = semantic_diff(&ddl, &live);
        assert!(differences.is_empty(), "{differences:#?}");
    }
}

fn parity_schema(mut schema: sifr_sql_contract::SchemaIr) -> sifr_sql_contract::SchemaIr {
    schema.objects.retain(|identity, _| {
        identity.as_str() == "public"
            || identity.as_str() == "public.parity_users"
            || identity.as_str().starts_with("public.parity_users.")
            || identity.as_str().starts_with("public.parity_users_")
            || identity.as_str() == "public.parity_owned_sequence"
            || identity.as_str() == "public.parity_user_view"
            || identity.as_str().starts_with("public.parity_user_view.")
    });
    schema
}

fn ddl_parity_schema(provider: ProviderIdentity) -> sifr_sql_contract::SchemaIr {
    let response = PostgresCompilerComponent::new(LibpgQueryParser).execute(
        PostgresComponentRequest::NormalizeSchema {
            provider: provider.clone(),
            server_major: 18,
            documents: vec![(
                "parity.sql".to_string(),
                "CREATE TABLE parity_users (\
                    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY, \
                    name text NOT NULL, \
                    score integer CHECK (score >= 0)\
                 ); \
                 CREATE VIEW parity_user_view AS \
                    SELECT id, name, score FROM parity_users; \
                 CREATE SEQUENCE parity_owned_sequence AS integer INCREMENT 5 \
                    MINVALUE 10 MAXVALUE 1000 START 20 CACHE 3 CYCLE; \
                 ALTER SEQUENCE parity_owned_sequence OWNED BY parity_users.id;"
                    .to_string(),
            )],
        },
    );
    let PostgresComponentResponse::Schema(output) = response else {
        panic!("DDL parity schema must normalize");
    };
    normalize_schema(provider, output.dialect, output.documents).expect("DDL schema")
}

fn authority(schema: sifr_sql_contract::SchemaIr) -> sifr_sql_contract::ProfileAuthority {
    build_profile_authority(SchemaProfile {
        package_id: "app@1.0.0#qualification".to_string(),
        name: "app".to_string(),
        source_files: BTreeSet::from(["db/schema.sql".to_string()]),
        source_fingerprints: BTreeMap::from([("db/schema.sql".to_string(), "b".repeat(64))]),
        evidence: SchemaEvidence::Introspection,
        strictness: SchemaStrictness::Exact,
        pooling: PoolingMode::Session,
        session: SessionContract::default(),
        accepted_signers: BTreeSet::new(),
        capabilities: sifr_sql_postgresql::postgresql_capabilities(),
        schema,
    })
    .expect("authority")
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

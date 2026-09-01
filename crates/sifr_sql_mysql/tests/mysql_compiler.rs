#![allow(clippy::expect_used)]

use semver::Version;
use sifr_compiler_component::{
    AnalysisContext, COMPONENT_PROTOCOL_MAJOR, ComponentHost, ComponentHostLimits,
    ComponentIdentity, ComponentRegistration, ContextArtifact, EmbeddedAnalysisRequest, PlanKind,
    ProtocolRange, SourceSpan, TemplatePart,
};
use sifr_sql_contract::{
    ProviderIdentity, SchemaDocumentKind, SchemaObjectKind, SchemaSourceInput, SessionContract,
    normalize_schema, schema_fingerprint, schema_normalization_from_response,
    schema_normalization_request, schema_source_fingerprint,
};
use sifr_sql_mysql::{
    MYSQL_SCHEMA_ARTIFACT_KIND, MysqlAnalyzer, MysqlEditorFacts, MysqlParser, MysqlSchemaOptions,
    MysqlServerSeries, SUPPORTED_MYSQL_SERIES, component_artifact_path, component_registration,
    execute_embedded_request, normalize_mysql_documents, provider_diagnostics,
};
use std::collections::{BTreeMap, BTreeSet};

fn parser(series: MysqlServerSeries) -> MysqlParser {
    MysqlParser::new(
        series,
        ["STRICT_TRANS_TABLES"],
        "utf8mb4",
        "utf8mb4_0900_ai_ci",
    )
    .expect("valid parser")
}

fn options() -> MysqlSchemaOptions {
    MysqlSchemaOptions {
        default_database: "app".to_string(),
        default_character_set: "utf8mb4".to_string(),
        default_collation: "utf8mb4_0900_ai_ci".to_string(),
        sql_modes: BTreeSet::from(["STRICT_TRANS_TABLES".to_string()]),
        extensions: BTreeSet::new(),
    }
}

fn provider() -> ProviderIdentity {
    ProviderIdentity {
        package_id: "sifr-sql-mysql@0.0.0#qualification".to_string(),
        package_version: Version::new(0, 0, 0),
        package_source: "workspace:crates/sifr_sql_mysql".to_string(),
        package_graph_digest: "a".repeat(64),
        compiler_components: BTreeMap::from([("mysql".to_string(), "b".repeat(64))]),
    }
}

fn schema(ddl: &str) -> sifr_sql_contract::SchemaIr {
    schema_for_series(MysqlServerSeries::new(8, 4), ddl)
}

fn schema_for_series(series: MysqlServerSeries, ddl: &str) -> sifr_sql_contract::SchemaIr {
    let parser = parser(series);
    let output = normalize_mysql_documents(
        provider(),
        &parser,
        &options(),
        vec![("db/schema.mysql.sql".to_string(), ddl.to_string())],
    )
    .expect("normalize MySQL DDL");
    normalize_schema(provider(), output.dialect, output.documents).expect("SchemaIR")
}

#[test]
fn supported_series_own_query_and_ddl_parsing() {
    for series in [
        MysqlServerSeries::new(8, 4),
        MysqlServerSeries::new(9, 7),
        MysqlServerSeries::new(26, 7),
    ] {
        let parser = parser(series);
        assert!(
            parser
                .parse("SELECT id FROM users WHERE id = ? LIMIT 1")
                .is_ok()
        );
        assert!(parser.parse(
            "CREATE TABLE users (id BIGINT UNSIGNED PRIMARY KEY, slug VARCHAR(64) NOT NULL, normalized VARCHAR(64) GENERATED ALWAYS AS (LOWER(slug)) STORED, UNIQUE KEY users_slug (slug)) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci"
        ).is_ok());
        assert!(
            parser
                .parse("INSERT INTO users(id) VALUES (?) ON DUPLICATE KEY UPDATE id = ?")
                .is_ok()
        );
    }
}

#[test]
fn every_checked_in_mysql_component_executes_in_the_capability_free_host() {
    for series in SUPPORTED_MYSQL_SERIES {
        let schema = schema_for_series(
            series,
            "CREATE TABLE users(id BIGINT PRIMARY KEY, email VARCHAR(255) NOT NULL UNIQUE)",
        );
        let fingerprint = schema_fingerprint(&schema)
            .expect("schema fingerprint")
            .as_str()
            .to_string();
        let registration = component_registration(series).expect("component registration");
        let bytes = std::fs::read(component_artifact_path(series)).expect("component artifact");
        let request = EmbeddedAnalysisRequest {
            protocol_major: COMPONENT_PROTOCOL_MAJOR,
            component: registration.identity.clone(),
            provider_diagnostics: registration.diagnostics.clone(),
            compiler_semantic_version: "0.0.0".to_string(),
            parts: vec![TemplatePart::Static {
                text: "SELECT id FROM users ORDER BY id LIMIT 1".to_string(),
                span: SourceSpan {
                    document: "src/component.sifr".to_string(),
                    start: 0,
                    end: 41,
                },
            }],
            holes: Vec::new(),
            context: AnalysisContext {
                schema_profile: Some("app.Schema".to_string()),
                schema_fingerprint: Some(fingerprint.clone()),
                semantic_profile: BTreeMap::new(),
                imported_signatures: Vec::new(),
                artifacts: vec![ContextArtifact {
                    kind: MYSQL_SCHEMA_ARTIFACT_KIND.to_string(),
                    identity: "app.Schema".to_string(),
                    format_version: 1,
                    fingerprint,
                    payload: serde_json::to_vec(&schema).expect("schema payload"),
                }],
            },
            plan_kind: PlanKind::Expression,
        };
        let mut host = ComponentHost::new(
            ComponentHostLimits {
                fuel: 100_000_000,
                ..ComponentHostLimits::default()
            },
            None,
        )
        .expect("component host");
        let run = host
            .analyze(&registration, &bytes, &request)
            .unwrap_or_else(|failure| panic!("MySQL {series:?} component failed: {failure}"));
        assert_eq!(
            run.response.plan.provider_identity,
            registration.identity.processor
        );
        assert!(run.response.plan.diagnostics.is_empty());
        assert!(!run.response.plan.operations.is_empty());
    }
}

#[test]
fn ansi_quotes_changes_the_lexical_contract() {
    let ordinary = parser(MysqlServerSeries::new(8, 4));
    let ansi = MysqlParser::new(
        MysqlServerSeries::new(8, 4),
        ["ANSI_QUOTES", "STRICT_TRANS_TABLES"],
        "utf8mb4",
        "utf8mb4_0900_ai_ci",
    )
    .expect("ANSI parser");
    assert_ne!(
        ordinary
            .normalize("SELECT \"name\" FROM users")
            .expect("string"),
        ansi.normalize("SELECT \"name\" FROM users")
            .expect("identifier")
    );
}

#[test]
fn schema_models_unsigned_generated_collation_and_content_constraint_ids() {
    let first = schema(
        "CREATE TABLE users (id BIGINT UNSIGNED PRIMARY KEY, email VARCHAR(255) NOT NULL COLLATE utf8mb4_0900_ai_ci, slug VARCHAR(255) GENERATED ALWAYS AS (LOWER(email)) STORED, CONSTRAINT users_email UNIQUE (email))",
    );
    let second = schema(
        "CREATE TABLE users (slug VARCHAR(255) GENERATED ALWAYS AS (LOWER(email)) STORED, email VARCHAR(255) NOT NULL COLLATE utf8mb4_0900_ai_ci, id BIGINT UNSIGNED PRIMARY KEY, CONSTRAINT users_email UNIQUE (email))",
    );
    let constraints = |schema: &sifr_sql_contract::SchemaIr| {
        schema
            .objects
            .values()
            .filter(|object| {
                matches!(
                    object.kind,
                    SchemaObjectKind::PrimaryKey | SchemaObjectKind::UniqueConstraint
                )
            })
            .map(|object| object.identity.clone())
            .collect::<BTreeSet<_>>()
    };
    assert_eq!(constraints(&first), constraints(&second));
    assert!(first.dialect.modes.contains("STRICT_TRANS_TABLES"));
    assert!(first.dialect.modes.contains("collation:utf8mb4_0900_ai_ci"));
    assert_ne!(schema_fingerprint(&first), schema_fingerprint(&second));
}

#[test]
fn schema_rejects_character_set_drift_and_uses_the_server_charset_default() {
    let parser = parser(MysqlServerSeries::new(8, 4));
    let mut mismatched = options();
    mismatched.default_character_set = "latin1".to_string();
    assert!(
        normalize_mysql_documents(
            provider(),
            &parser,
            &mismatched,
            vec![(
                "db/mismatch.sql".to_string(),
                "CREATE TABLE users (id BIGINT)".to_string()
            )],
        )
        .is_err()
    );

    let schema = schema("CREATE TABLE latin_users (name VARCHAR(32)) CHARACTER SET latin1");
    for identity in ["app.latin_users", "app.latin_users.name"] {
        assert_eq!(
            schema.objects[&sifr_sql_contract::ObjectId::new(identity)]
                .semantic
                .get("collation"),
            Some(&sifr_sql_contract::SemanticValue::Text(
                "latin1_swedish_ci".to_string()
            ))
        );
    }
}

#[test]
fn analyzer_accounts_for_relations_columns_parameters_and_conflict_capability() {
    let schema = schema(
        "CREATE TABLE users (id BIGINT UNSIGNED PRIMARY KEY, email VARCHAR(255) NOT NULL UNIQUE)",
    );
    let parser = parser(MysqlServerSeries::new(8, 4));
    let analyzer = MysqlAnalyzer::new(&parser, &schema).expect("analyzer");
    let read = analyzer
        .analyze_query("SELECT id, email FROM users WHERE email = ? LIMIT 1")
        .expect("read analysis");
    assert_eq!(read.parameters.len(), 1);
    assert_eq!(read.result_fields.len(), 2);
    assert_eq!(
        read.accessed_objects,
        BTreeSet::from([
            sifr_sql_contract::ObjectId::new("app.users"),
            sifr_sql_contract::ObjectId::new("app.users.id"),
            sifr_sql_contract::ObjectId::new("app.users.email"),
        ])
    );
    let write = analyzer
        .analyze_query(
            "INSERT INTO users(id, email) VALUES (?, ?) ON DUPLICATE KEY UPDATE email = ?",
        )
        .expect("write analysis");
    assert!(
        write
            .required_capabilities
            .contains("sql.mysql.write.conflict")
    );
}

#[test]
fn editor_recovery_is_non_authoritative_and_settings_sensitive() {
    let parser = parser(MysqlServerSeries::new(8, 4));
    let facts = MysqlEditorFacts::analyze(&parser, "SELECT id FROM users WHERE");
    assert!(!facts.recovery.compile_authority);
    assert!(facts.documentation_base.contains("8.4"));
    let changed = MysqlParser::new(
        MysqlServerSeries::new(8, 4),
        ["ANSI_QUOTES", "STRICT_TRANS_TABLES"],
        "utf8mb4",
        "utf8mb4_0900_ai_ci",
    )
    .expect("changed parser");
    assert_ne!(
        facts.semantic_settings_fingerprint,
        MysqlEditorFacts::analyze(&changed, "SELECT id FROM users WHERE")
            .semantic_settings_fingerprint
    );
}

#[test]
fn mysql_independently_normalizes_portable_requirement_sources() {
    let source =
        b"CREATE TABLE users (id BIGINT UNSIGNED PRIMARY KEY, email VARCHAR(255) NOT NULL UNIQUE)";
    let sources = vec![SchemaSourceInput {
        document: "requirements/has_users.mysql.sql".to_string(),
        kind: SchemaDocumentKind::SqlDdl,
        fingerprint: schema_source_fingerprint(source),
        contents: source.to_vec(),
    }];
    let registration = ComponentRegistration {
        identity: ComponentIdentity {
            package: "sifr-sql-mysql".to_string(),
            processor: "sifr.sql.mysql.schema".to_string(),
            version: Version::new(0, 0, 0),
            sha256: "c".repeat(64),
        },
        protocol: ProtocolRange {
            minimum: COMPONENT_PROTOCOL_MAJOR,
            maximum: COMPONENT_PROTOCOL_MAJOR,
        },
        artifact: "components/mysql-8.4.wasm".to_string(),
        diagnostics: provider_diagnostics(),
    };
    let session = SessionContract {
        search_path: vec!["app".to_string()],
        sql_modes: BTreeSet::from(["STRICT_TRANS_TABLES".to_string()]),
        collation: Some("utf8mb4_0900_ai_ci".to_string()),
        character_set: Some("utf8mb4".to_string()),
        ..SessionContract::default()
    };
    let request = schema_normalization_request(
        &registration,
        "0.0.0",
        "portable::has_users::mysql",
        "8.4",
        &session,
        &BTreeSet::new(),
        &sources,
    )
    .expect("normalization request");
    let response = execute_embedded_request(request).expect("MySQL component response");
    let normalized = schema_normalization_from_response(provider(), &sources, &response)
        .expect("portable MySQL schema");
    assert!(
        normalized
            .schema
            .objects
            .contains_key(&sifr_sql_contract::ObjectId::new("app.users"))
    );
    assert!(normalized.capabilities.contains("sql.query.select"));
}

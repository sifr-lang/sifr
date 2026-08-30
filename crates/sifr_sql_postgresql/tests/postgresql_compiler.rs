#![allow(clippy::expect_used, clippy::unwrap_used)]

use semver::Version;
use sifr_sql_contract::{
    Cardinality, DatabaseType, ProviderIdentity, QueryEffect, SchemaDocument, SchemaDocumentKind,
    SchemaObject, SchemaObjectKind, SemanticValue, normalize_schema,
};
use sifr_sql_postgresql::{
    LibpgQueryParser, POSTGRESQL_QUERY_OPERATION, PostgresCompilerComponent,
    PostgresComponentRequest, PostgresComponentResponse, PostgresDiagnosticCode, PostgresParser,
    SUPPORTED_POSTGRESQL_MAJORS, StatementKind, component_registration, embedded_sources,
    into_embedded_response, rewrite_parameter_slots,
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn exact_libpg_query_sources_and_registration_cover_every_supported_major() {
    let sources = embedded_sources().expect("checked-in source manifest");
    assert_eq!(
        sources
            .iter()
            .map(|source| source.server_major)
            .collect::<Vec<_>>(),
        SUPPORTED_POSTGRESQL_MAJORS
    );
    for source in sources {
        assert_eq!(source.commit.len(), 40);
        assert_eq!(source.archive_sha256.len(), 64);
        assert!(source.tag.starts_with(&source.server_major.to_string()));
        let registration = component_registration(source.server_major, "a".repeat(64)).unwrap();
        assert!(
            registration
                .identity
                .processor
                .starts_with(POSTGRESQL_QUERY_OPERATION)
        );
        assert!(
            registration
                .artifact
                .ends_with(&format!("postgresql-{}.wasm", source.server_major))
        );
        assert_eq!(registration.diagnostics.declarations.len(), 11);
    }
}

#[test]
fn fragment_parameter_rewrite_preserves_postgresql_lexical_boundaries() {
    let source = "id = $1 AND note = '$2' AND body = $tag$ $3 $tag$ /* $4 */ -- $5\nOR x = $2";
    assert_eq!(
        rewrite_parameter_slots(source, 3).unwrap(),
        "id = $4 AND note = '$2' AND body = $tag$ $3 $tag$ /* $4 */ -- $5\nOR x = $5"
    );
    assert!(rewrite_parameter_slots("x = $0", 1).is_err());
}

#[test]
fn libpg_query_maps_raw_nodes_to_owned_select_write_and_ddl_nodes() {
    let parser = LibpgQueryParser;
    let select = parser
        .parse(
            "SELECT u.id, lower(u.name) AS normalized FROM public.users AS u \
             WHERE u.id = $1 UNION ALL SELECT id, name FROM archived_users",
        )
        .unwrap();
    assert!(matches!(select[0].kind, StatementKind::Select(_)));

    let insert = parser
        .parse(
            "INSERT INTO users(id, name) VALUES ($1, $2) \
             ON CONFLICT(id) DO UPDATE SET name = excluded.name RETURNING id",
        )
        .unwrap();
    assert!(matches!(insert[0].kind, StatementKind::Insert(_)));

    let ddl = parser
        .parse(
            "CREATE TABLE public.users (\
               id bigint PRIMARY KEY,\
               name text NOT NULL,\
               generated text GENERATED ALWAYS AS (name || id::text) STORED\
             )",
        )
        .unwrap();
    let StatementKind::CreateTable(table) = &ddl[0].kind else {
        panic!("expected provider-owned CREATE TABLE node");
    };
    assert_eq!(table.columns.len(), 3);
    assert!(table.columns[2].generated);
}

#[test]
fn ddl_normalization_and_query_analysis_share_one_schema_authority() {
    let component = PostgresCompilerComponent::new(LibpgQueryParser);
    let server_major = LibpgQueryParser.server_major();
    let response = component.execute(PostgresComponentRequest::NormalizeSchema {
        provider: provider(),
        server_major,
        documents: vec![(
            "db/schema.sql".to_string(),
            "CREATE TABLE public.teams (id integer PRIMARY KEY, name text NOT NULL);\
             CREATE TABLE public.users (\
               id bigint PRIMARY KEY,\
               name text NOT NULL UNIQUE,\
               team_id integer REFERENCES public.teams(id),\
               nickname text,\
               generated text GENERATED ALWAYS AS (name || id::text) STORED,\
               CHECK (name <> '')\
             );\
             CREATE UNIQUE INDEX users_name_idx ON public.users(name);\
             CREATE VIEW public.user_names AS SELECT id, name FROM public.users;\
             CREATE MATERIALIZED VIEW public.team_names AS SELECT id, name FROM public.teams;\
             CREATE SEQUENCE public.audit_sequence;\
             CREATE TYPE public.mood AS ENUM ('ok', 'sad');\
             CREATE DOMAIN public.label AS text NOT NULL;"
                .to_string(),
        )],
    });
    let PostgresComponentResponse::Schema(schema_output) = response else {
        panic!("DDL must normalize: {response:?}");
    };
    let schema = normalize_schema(provider(), schema_output.dialect, schema_output.documents)
        .expect("normalized SchemaIR");
    assert_eq!(
        schema
            .objects
            .values()
            .filter(|object| object.kind == SchemaObjectKind::Table)
            .count(),
        2
    );
    let kinds = schema
        .objects
        .values()
        .map(|object| object.kind)
        .collect::<BTreeSet<_>>();
    for kind in [
        SchemaObjectKind::CheckConstraint,
        SchemaObjectKind::Column,
        SchemaObjectKind::Domain,
        SchemaObjectKind::Enum,
        SchemaObjectKind::ForeignKey,
        SchemaObjectKind::Index,
        SchemaObjectKind::MaterializedView,
        SchemaObjectKind::Namespace,
        SchemaObjectKind::PrimaryKey,
        SchemaObjectKind::Sequence,
        SchemaObjectKind::Table,
        SchemaObjectKind::UniqueConstraint,
        SchemaObjectKind::View,
    ] {
        assert!(kinds.contains(&kind), "missing normalized object {kind:?}");
    }

    let query = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema: schema.clone(),
        source: "SELECT u.id, lower(u.name) AS normalized, u.nickname \
                 FROM public.users AS u WHERE u.id = $1 ORDER BY u.id LIMIT 1"
            .to_string(),
        sifr_document: "src/queries.sifr".to_string(),
        sifr_start: 40,
        sifr_end: 180,
    });
    let PostgresComponentResponse::Query(analysis) = query else {
        panic!("query must analyze");
    };
    assert_eq!(analysis.parameters.len(), 1);
    assert!(matches!(
        analysis.parameters[0].database_type,
        DatabaseType::Integer { .. }
    ));
    assert_eq!(analysis.result_fields.len(), 3);
    assert!(matches!(
        analysis.result_fields[0].sifr_type,
        sifr_sql_contract::SifrType::FixedInteger { .. }
    ));
    assert_eq!(
        analysis.result_fields[2].nullability,
        sifr_sql_contract::Nullability::Nullable
    );
    assert_eq!(analysis.cardinality, Cardinality::AT_MOST_ONE);
    assert_eq!(analysis.effects.effect, QueryEffect::Read);
    assert!(analysis.semantic_flags.contains("deterministic-order"));

    let embedded = into_embedded_response(
        server_major,
        Some("app.Schema".to_string()),
        &schema_output_fingerprint(&schema),
        &PostgresComponentResponse::Query(analysis),
    )
    .unwrap();
    assert_eq!(embedded.plan.operations.len(), 1);
    assert_eq!(embedded.plan.schema_identity.as_deref(), Some("app.Schema"));
}

#[test]
fn aliases_correlations_set_operations_and_parameter_codecs_are_exact() {
    let component = PostgresCompilerComponent::new(LibpgQueryParser);
    let server_major = LibpgQueryParser.server_major();
    let schema = schema_for_semantics(&component, server_major);
    let correlated = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema: schema.clone(),
        source: "SELECT u.id AS user_id, (SELECT t.name FROM teams AS t \
                 WHERE t.id = u.team_id LIMIT 1) AS team_name \
                 FROM users AS u WHERE u.id = $1 ORDER BY user_id LIMIT 1"
            .to_string(),
        sifr_document: "src/semantic.sifr".to_string(),
        sifr_start: 20,
        sifr_end: 170,
    });
    let PostgresComponentResponse::Query(correlated) = correlated else {
        panic!("correlated query must analyze: {correlated:?}");
    };
    assert_eq!(correlated.parameters.len(), 1);
    assert!(correlated.parameters[0].codec.as_str().contains("int8"));
    assert_eq!(correlated.result_fields[0].name, "user_id");
    assert_eq!(
        correlated.result_fields[1].nullability,
        sifr_sql_contract::Nullability::Nullable
    );
    assert_eq!(correlated.cardinality, Cardinality::AT_MOST_ONE);

    let set = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema,
        source: "SELECT id AS value FROM users UNION ALL \
                 SELECT id AS value FROM users ORDER BY value LIMIT 1"
            .to_string(),
        sifr_document: "src/semantic.sifr".to_string(),
        sifr_start: 180,
        sifr_end: 290,
    });
    let PostgresComponentResponse::Query(set) = set else {
        panic!("set operation must analyze");
    };
    assert_eq!(set.result_fields[0].name, "value");
    assert_eq!(set.cardinality, Cardinality::AT_MOST_ONE);
    assert!(set.semantic_flags.contains("set-union"));
    assert!(set.semantic_flags.contains("deterministic-order"));
}

#[test]
fn semantic_failures_keep_virtual_sifr_and_schema_spans() {
    let component = PostgresCompilerComponent::new(LibpgQueryParser);
    let server_major = LibpgQueryParser.server_major();
    let schema = schema_for_semantics(&component, server_major);
    let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema: schema.clone(),
        source: "SELECT u.missing FROM users AS u".to_string(),
        sifr_document: "src/errors.sifr".to_string(),
        sifr_start: 30,
        sifr_end: 66,
    });
    let PostgresComponentResponse::Diagnostic(diagnostic) = response else {
        panic!("unknown column must fail");
    };
    assert_eq!(
        diagnostic.primary.kind,
        sifr_sql_postgresql::PostgresSpanKind::VirtualSql
    );
    assert!(
        diagnostic
            .related
            .iter()
            .any(|span| span.kind == sifr_sql_postgresql::PostgresSpanKind::Sifr)
    );
    assert!(
        diagnostic
            .related
            .iter()
            .any(|span| span.kind == sifr_sql_postgresql::PostgresSpanKind::Schema)
    );

    for source in [
        "SELECT id AS duplicate, name AS duplicate FROM users",
        "SELECT id FROM users WHERE id = $1 AND name = $1",
        "SELECT id FROM users WHERE id = $2",
    ] {
        let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
            schema: schema.clone(),
            source: source.to_string(),
            sifr_document: "src/errors.sifr".to_string(),
            sifr_start: 0,
            sifr_end: u32::try_from(source.len()).unwrap(),
        });
        assert!(matches!(response, PostgresComponentResponse::Diagnostic(_)));
    }
}

#[test]
fn update_and_delete_returning_preserve_write_effects() {
    let component = PostgresCompilerComponent::new(LibpgQueryParser);
    let server_major = LibpgQueryParser.server_major();
    let schema = schema_for_writes(&component, server_major);
    for source in [
        "UPDATE users SET name = $1 WHERE id = $2 RETURNING id, nickname",
        "DELETE FROM users WHERE id = $1 RETURNING id",
    ] {
        let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
            schema: schema.clone(),
            source: source.to_string(),
            sifr_document: "src/write.sifr".to_string(),
            sifr_start: 0,
            sifr_end: u32::try_from(source.len()).unwrap(),
        });
        let PostgresComponentResponse::Query(analysis) = response else {
            panic!("write must analyze: {response:?}");
        };
        assert_eq!(analysis.effects.effect, QueryEffect::Write);
        assert_eq!(analysis.effects.affected_objects.len(), 1);
        assert!(!analysis.result_fields.is_empty());
    }
}

fn schema_output_fingerprint(schema: &sifr_sql_contract::SchemaIr) -> String {
    sifr_sql_contract::schema_fingerprint(schema)
        .unwrap()
        .as_str()
        .to_string()
}

#[test]
fn writes_enforce_required_generated_conflict_and_returning_contracts() {
    let component = PostgresCompilerComponent::new(LibpgQueryParser);
    let server_major = LibpgQueryParser.server_major();
    let schema = schema_for_writes(&component, server_major);

    let valid = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema: schema.clone(),
        source: "INSERT INTO users(id, name) VALUES ($1, $2) \
                 ON CONFLICT(id) DO UPDATE SET name = excluded.name RETURNING id"
            .to_string(),
        sifr_document: "src/write.sifr".to_string(),
        sifr_start: 10,
        sifr_end: 120,
    });
    let PostgresComponentResponse::Query(valid) = valid else {
        panic!("valid write must analyze: {valid:?}");
    };
    assert_eq!(valid.parameters.len(), 2);
    assert_eq!(valid.effects.effect, QueryEffect::Write);
    assert_eq!(valid.result_fields.len(), 1);

    for source in [
        "INSERT INTO users(id) VALUES ($1)",
        "INSERT INTO users(id, name, generated) VALUES ($1, $2, $3)",
        "INSERT INTO users(id, name) VALUES ($1, $2) ON CONFLICT(name) DO NOTHING",
    ] {
        let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
            schema: schema.clone(),
            source: source.to_string(),
            sifr_document: "src/write.sifr".to_string(),
            sifr_start: 0,
            sifr_end: u32::try_from(source.len()).unwrap(),
        });
        let PostgresComponentResponse::Diagnostic(diagnostic) = response else {
            panic!("invalid write must produce a diagnostic");
        };
        assert_eq!(diagnostic.code, PostgresDiagnosticCode::InvalidWrite);
        assert!(
            diagnostic
                .related
                .iter()
                .any(|span| { span.kind == sifr_sql_postgresql::PostgresSpanKind::Sifr })
        );
    }
}

#[test]
fn metadata_catalog_resolves_functions_operators_casts_and_schema_spans() {
    let server_major = LibpgQueryParser.server_major();
    let base = DatabaseType::Text {
        fixed: false,
        max_characters: None,
    };
    let document = SchemaDocument {
        kind: SchemaDocumentKind::ProviderMetadata,
        document: "catalog.json".to_string(),
        objects: vec![SchemaObject {
            identity: sifr_sql_contract::ObjectId::new("public.custom_lower"),
            kind: SchemaObjectKind::Function,
            semantic: BTreeMap::from([
                (
                    "arguments".to_string(),
                    SemanticValue::List(vec![SemanticValue::Text(
                        serde_json::to_string(&base).unwrap(),
                    )]),
                ),
                (
                    "result".to_string(),
                    SemanticValue::Text(serde_json::to_string(&base).unwrap()),
                ),
                ("strict".to_string(), SemanticValue::Bool(true)),
                ("aggregate".to_string(), SemanticValue::Bool(false)),
                ("result-nullable".to_string(), SemanticValue::Bool(false)),
            ]),
            dependencies: BTreeSet::new(),
            source: Some(sifr_sql_contract::SchemaSourceLocation {
                document: "catalog.json".to_string(),
                start: 12,
                end: 45,
            }),
        }],
    };
    let schema = normalize_schema(
        provider(),
        sifr_sql_contract::DialectIdentity {
            family: "postgresql".to_string(),
            server_version: server_major.to_string(),
            modes: BTreeSet::new(),
            features: BTreeSet::new(),
        },
        [document],
    )
    .unwrap();
    let catalog = sifr_sql_postgresql::PostgresCatalog::from_schema(
        &schema,
        sifr_sql_postgresql::PostgresTypeRegistry::new(server_major),
    )
    .unwrap();
    assert_eq!(catalog.functions(&["custom_lower".to_string()]).len(), 1);
}

fn schema_for_writes(
    component: &PostgresCompilerComponent<LibpgQueryParser>,
    server_major: u16,
) -> sifr_sql_contract::SchemaIr {
    let response = component.execute(PostgresComponentRequest::NormalizeSchema {
        provider: provider(),
        server_major,
        documents: vec![(
            "db/write.sql".to_string(),
            "CREATE TABLE public.users (\
               id bigint PRIMARY KEY,\
               name text NOT NULL,\
               nickname text,\
               generated text GENERATED ALWAYS AS (name || id::text) STORED\
             );"
            .to_string(),
        )],
    });
    let PostgresComponentResponse::Schema(output) = response else {
        panic!("write schema must normalize: {response:?}");
    };
    normalize_schema(provider(), output.dialect, output.documents).unwrap()
}

fn schema_for_semantics(
    component: &PostgresCompilerComponent<LibpgQueryParser>,
    server_major: u16,
) -> sifr_sql_contract::SchemaIr {
    let response = component.execute(PostgresComponentRequest::NormalizeSchema {
        provider: provider(),
        server_major,
        documents: vec![(
            "db/semantic.sql".to_string(),
            "CREATE TABLE public.teams (id integer PRIMARY KEY, name text NOT NULL);\
             CREATE TABLE public.users (\
               id bigint PRIMARY KEY,\
               name text NOT NULL,\
               team_id integer NOT NULL REFERENCES public.teams(id),\
               nickname text\
             );"
            .to_string(),
        )],
    });
    let PostgresComponentResponse::Schema(output) = response else {
        panic!("semantic schema must normalize: {response:?}");
    };
    normalize_schema(provider(), output.dialect, output.documents).unwrap()
}

fn provider() -> ProviderIdentity {
    ProviderIdentity {
        package_id: "sifr-sql-postgresql@0.0.0#workspace".to_string(),
        package_version: Version::new(0, 0, 0),
        package_source: "workspace:crates/sifr_sql_postgresql".to_string(),
        package_graph_digest: "b".repeat(64),
        compiler_components: BTreeMap::from([(
            "sifr.sql.postgresql.sql".to_string(),
            "c".repeat(64),
        )]),
    }
}
